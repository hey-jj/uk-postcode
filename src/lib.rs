//! Parse, validate, and normalize UK postcodes.
//!
//! This crate checks the *shape* of a postcode. It does not look the postcode up
//! against a real list, so a string with a valid shape may still not name a real
//! place. A full check needs the Royal Mail or Ordnance Survey postcode data.
//!
//! Every function takes a `&str`. Validators return `bool`. Parsers return
//! `Option<String>` and yield `None` when the input is not a valid postcode.
//! [`parse`] returns a [`Postcode`], either [`Postcode::Valid`] with every
//! component or [`Postcode::Invalid`].
//!
//! # Examples
//!
//! ```
//! use uk_postcode::{is_valid, to_normalised, parse};
//!
//! assert!(is_valid("SW1A 2AA"));
//! assert_eq!(to_normalised("sw1a2aa").as_deref(), Some("SW1A 2AA"));
//!
//! let p = parse("Sw1A 2aa");
//! let valid = p.valid().expect("valid postcode");
//! assert_eq!(valid.outcode, "SW1A");
//! assert_eq!(valid.area, "SW");
//! ```
//!
//! # Components
//!
//! A postcode splits into an outward code and an inward code, here written with
//! `A` for a letter and `9` for a digit.
//!
//! | Schema     | outcode | incode | area | district | sub-district | sector   | unit |
//! |------------|---------|--------|------|----------|--------------|----------|------|
//! | `AA9A 9AA` | `AA9A`  | `9AA`  | `AA` | `AA9`    | `AA9A`       | `AA9A 9` | `AA` |
//! | `A9A 9AA`  | `A9A`   | `9AA`  | `A`  | `A9`     | `A9A`        | `A9A 9`  | `AA` |
//! | `A9 9AA`   | `A9`    | `9AA`  | `A`  | `A9`     | none         | `A9 9`   | `AA` |
//! | `A99 9AA`  | `A99`   | `9AA`  | `A`  | `A99`    | none         | `A99 9`  | `AA` |
//! | `AA9 9AA`  | `AA9`   | `9AA`  | `AA` | `AA9`    | none         | `AA9 9`  | `AA` |
//! | `AA99 9AA` | `AA99`  | `9AA`  | `AA` | `AA99`   | none         | `AA99 9` | `AA` |

#![forbid(unsafe_code)]
#![warn(missing_docs)]

/// The result of parsing a postcode candidate.
///
/// A valid candidate carries every component in [`ValidPostcode`]. An invalid one
/// carries nothing. The two cases are separate variants, so a "valid" postcode
/// with missing components cannot exist.
///
/// Build one with [`parse`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub enum Postcode {
    /// The candidate had a valid postcode shape. Holds every component.
    Valid(ValidPostcode),
    /// The candidate did not have a valid postcode shape.
    Invalid,
}

impl Postcode {
    /// Return `true` when the candidate parsed to a valid postcode.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        matches!(self, Postcode::Valid(_))
    }

    /// Return the components when valid, or `None` when invalid.
    #[must_use]
    pub fn valid(&self) -> Option<&ValidPostcode> {
        match self {
            Postcode::Valid(p) => Some(p),
            Postcode::Invalid => None,
        }
    }
}

/// The components of a valid postcode.
///
/// Every field is filled except `sub_district`, which is `Some` only when the
/// outcode ends in a letter.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct ValidPostcode {
    /// Normalised postcode, uppercased with a single space, for example `SW1A 2AA`.
    pub postcode: String,
    /// Inward code, for example `2AA`.
    pub incode: String,
    /// Outward code, for example `SW1A`.
    pub outcode: String,
    /// Postcode area, the leading one or two letters, for example `SW`.
    pub area: String,
    /// Postcode district, for example `SW1`.
    pub district: String,
    /// Postcode sub-district, present only when the outcode ends in a letter,
    /// for example `SW1A`.
    pub sub_district: Option<String>,
    /// Postcode sector, the outcode plus the first incode digit, for example `SW1A 2`.
    pub sector: String,
    /// Postcode unit, the final two letters, for example `AA`.
    pub unit: String,
}

impl std::fmt::Display for ValidPostcode {
    /// Write the normalised postcode, for example `SW1A 2AA`.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.postcode)
    }
}

impl std::fmt::Display for Postcode {
    /// Write the normalised postcode when valid, or nothing when invalid.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Postcode::Valid(p) => p.fmt(f),
            Postcode::Invalid => Ok(()),
        }
    }
}

/// The error from parsing a string that is not a valid postcode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsePostcodeError;

impl std::fmt::Display for ParsePostcodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid postcode")
    }
}

impl std::error::Error for ParsePostcodeError {}

impl std::str::FromStr for Postcode {
    type Err = ParsePostcodeError;

    /// Parse a postcode candidate.
    ///
    /// Returns the parsed [`Postcode::Valid`] on success. Returns
    /// [`ParsePostcodeError`] when the input is not a valid postcode, so `?`
    /// works at the call site. A valid postcode round-trips through [`Display`].
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match parse(s) {
            Postcode::Invalid => Err(ParsePostcodeError),
            valid => Ok(valid),
        }
    }
}

/// Outcome of [`replace`]: the postcodes found and the rewritten text.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct ReplaceResult {
    /// Postcodes found in the text, in document order, with original casing and spacing.
    pub matches: Vec<String>,
    /// The text with every postcode replaced.
    pub result: String,
}

// --- character classes -----------------------------------------------------
//
// The shape uses ASCII letters and digits only. Case folds for ASCII letters,
// so these helpers do the same. They never look at non-ASCII bytes, which keeps
// behaviour identical to ASCII-only `[a-z]` classes.

#[inline]
fn is_letter(b: u8) -> bool {
    b.is_ascii_alphabetic()
}

#[inline]
fn is_digit(b: u8) -> bool {
    b.is_ascii_digit()
}

#[inline]
fn is_letter_or_digit(b: u8) -> bool {
    b.is_ascii_alphanumeric()
}

/// Whether `c` counts as whitespace for sanitising and shape checks.
///
/// The set covers the ASCII controls, the regular space, the no-break space, the
/// line and paragraph separators, the Unicode space marks, and the byte order
/// mark. Rust's own whitespace check leaves out the byte order mark, so the list
/// is spelled out here.
fn is_js_space(c: char) -> bool {
    matches!(
        c,
        '\u{0009}' // tab
            | '\u{000A}' // line feed
            | '\u{000B}' // vertical tab
            | '\u{000C}' // form feed
            | '\u{000D}' // carriage return
            | '\u{0020}' // space
            | '\u{00A0}' // no-break space
            | '\u{1680}' // ogham space mark
            | '\u{2000}'
            ..='\u{200A}' // en quad through hair space
            | '\u{2028}' // line separator
            | '\u{2029}' // paragraph separator
            | '\u{202F}' // narrow no-break space
            | '\u{205F}' // medium mathematical space
            | '\u{3000}' // ideographic space
            | '\u{FEFF}' // byte order mark
    )
}

/// Drop every whitespace character and uppercase the rest.
///
/// Removes all whitespace then uppercases. Uppercasing is ASCII only, which
/// matches the ASCII letter classes in the shape checks.
fn sanitize(s: &str) -> String {
    s.chars()
        .filter(|&c| !is_js_space(c))
        .map(|c| c.to_ascii_uppercase())
        .collect()
}

// --- shape checks ----------------------------------------------------------
//
// These match the anchored shape grammars against the raw input. They walk the
// bytes once. Non-ASCII bytes never satisfy a class, so any non-ASCII input
// fails, the same as the ASCII-only character classes.

/// Span of a leading outward code at the start of `bytes`, or `None`.
///
/// Matches `[a-z]{1,2}\d[a-z\d]?`. Returns the length without the optional final
/// character and the length with it. They differ only when that optional
/// character is present, which is where the grammar may backtrack.
fn outcode_span(bytes: &[u8]) -> Option<(usize, usize)> {
    let mut i = 0;
    if i < bytes.len() && is_letter(bytes[i]) {
        i += 1;
    } else {
        return None;
    }
    if i < bytes.len() && is_letter(bytes[i]) {
        i += 1;
    }
    if i < bytes.len() && is_digit(bytes[i]) {
        i += 1;
    } else {
        return None;
    }
    let min = i;
    if i < bytes.len() && is_letter_or_digit(bytes[i]) {
        i += 1;
    }
    Some((min, i))
}

/// Count leading ECMAScript whitespace bytes in `s`.
fn space_len(s: &str) -> usize {
    let mut consumed = 0;
    for c in s.chars() {
        if is_js_space(c) {
            consumed += c.len_utf8();
        } else {
            break;
        }
    }
    consumed
}

/// Whether `bytes` is exactly an inward code: `\d[a-z]{2}`.
fn is_incode(bytes: &[u8]) -> bool {
    bytes.len() == 3 && is_digit(bytes[0]) && is_letter(bytes[1]) && is_letter(bytes[2])
}

/// Match the postcode grammar `^[a-z]{1,2}\d[a-z\d]?\s*\d[a-z]{2}$`.
///
/// The check runs on the raw input with no trimming, so leading or trailing
/// whitespace fails.
///
/// The inward code is always the last three characters. The optional fourth
/// outcode character makes the split ambiguous, for example `RG45AY` could read
/// the `5` into the outcode or the incode. The grammar backtracks so the incode
/// wins. Anchoring the incode to the end and validating the remaining
/// prefix as an outcode reproduces that without a backtracking engine.
fn matches_postcode(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 3 {
        return false;
    }
    let split = bytes.len() - 3;
    if !is_incode(&bytes[split..]) {
        return false;
    }
    // Drop trailing whitespace between the outcode and the incode.
    let mut head_end = split;
    while head_end > 0 {
        let spaces = trailing_space_len(&s[..head_end]);
        if spaces == 0 {
            break;
        }
        head_end -= spaces;
    }
    let head = &s[..head_end];
    matches_outcode(head)
}

/// Count trailing ECMAScript whitespace bytes in `s`.
fn trailing_space_len(s: &str) -> usize {
    match s.chars().next_back() {
        Some(c) if is_js_space(c) => c.len_utf8(),
        _ => 0,
    }
}

/// Match `OUTCODE_REGEX`: `^[a-z]{1,2}\d[a-z\d]?$`.
fn matches_outcode(s: &str) -> bool {
    let bytes = s.as_bytes();
    match outcode_span(bytes) {
        Some((min, max)) => min == bytes.len() || max == bytes.len(),
        None => false,
    }
}

/// Split a sanitized postcode into outcode and incode.
///
/// The caller must pass a string that already passed [`matches_postcode`] and is
/// space stripped, so the inward code is always the final three characters.
fn split_sanitized(sanitized: &str) -> (&str, &str) {
    let len = sanitized.len();
    sanitized.split_at(len - 3)
}

/// Split an outcode into district and sub-district letter.
///
/// Matches `DISTRICT_SPLIT_REGEX`: `^([a-z]{1,2}\d)([a-z])$`. Returns the part
/// before the trailing letter when the outcode ends in a letter, else `None`.
fn district_split(outcode: &str) -> Option<&str> {
    let bytes = outcode.as_bytes();
    let n = bytes.len();
    if n < 3 {
        return None;
    }
    if !is_letter(bytes[n - 1]) {
        return None;
    }
    if !is_digit(bytes[n - 2]) {
        return None;
    }
    // Everything before the digit must be one or two letters.
    let head = &bytes[..n - 2];
    if head.is_empty() || head.len() > 2 {
        return None;
    }
    if head.iter().all(|&b| is_letter(b)) {
        Some(&outcode[..n - 1])
    } else {
        None
    }
}

// --- public validators -----------------------------------------------------

/// Return `true` when `postcode` has a valid postcode shape.
///
/// Tests the raw input. Leading or trailing whitespace fails. Whitespace between
/// the outward and inward codes is allowed, including none.
///
/// # Examples
///
/// ```
/// use uk_postcode::is_valid;
///
/// assert!(is_valid("L27 8XY"));
/// assert!(is_valid("NR103EZ"));
/// assert!(!is_valid("Definitely wrong"));
/// assert!(!is_valid(" SW1A 2AA"));
/// ```
#[must_use]
pub fn is_valid(postcode: &str) -> bool {
    matches_postcode(postcode)
}

/// Return `true` when `outcode` is a valid outward code.
///
/// Tests the raw input against `[a-z]{1,2}\d[a-z\d]?`. No whitespace is allowed.
///
/// # Examples
///
/// ```
/// use uk_postcode::is_valid_outcode;
///
/// assert!(is_valid_outcode("L27"));
/// assert!(is_valid_outcode("AA9A"));
/// assert!(!is_valid_outcode("BOGUS"));
/// ```
#[must_use]
pub fn is_valid_outcode(outcode: &str) -> bool {
    matches_outcode(outcode)
}

// --- public parsers --------------------------------------------------------

/// Return the normalised postcode: outcode, a single space, then incode.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_normalised;
///
/// assert_eq!(to_normalised("Sw1A 2aa").as_deref(), Some("SW1A 2AA"));
/// assert_eq!(to_normalised("L278XY").as_deref(), Some("L27 8XY"));
/// assert_eq!(to_normalised("Definitely wrong"), None);
/// ```
#[must_use]
pub fn to_normalised(postcode: &str) -> Option<String> {
    let outcode = to_outcode(postcode)?;
    let incode = to_incode(postcode)?;
    Some(format!("{outcode} {incode}"))
}

/// Return the outward code, uppercased and space stripped.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_outcode;
///
/// assert_eq!(to_outcode("L27 8XY").as_deref(), Some("L27"));
/// assert_eq!(to_outcode("AA9A 9AA").as_deref(), Some("AA9A"));
/// ```
#[must_use]
pub fn to_outcode(postcode: &str) -> Option<String> {
    if !is_valid(postcode) {
        return None;
    }
    let sanitized = sanitize(postcode);
    let (outcode, _) = split_sanitized(&sanitized);
    Some(outcode.to_string())
}

/// Return the inward code, uppercased.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_incode;
///
/// assert_eq!(to_incode("L27 8XY").as_deref(), Some("8XY"));
/// ```
#[must_use]
pub fn to_incode(postcode: &str) -> Option<String> {
    if !is_valid(postcode) {
        return None;
    }
    let sanitized = sanitize(postcode);
    let (_, incode) = split_sanitized(&sanitized);
    Some(incode.to_string())
}

/// Return the postcode area, the leading one or two letters.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_area;
///
/// assert_eq!(to_area("L27 8XY").as_deref(), Some("L"));
/// assert_eq!(to_area("NR10 3EZ").as_deref(), Some("NR"));
/// ```
#[must_use]
pub fn to_area(postcode: &str) -> Option<String> {
    if !is_valid(postcode) {
        return None;
    }
    let sanitized = sanitize(postcode);
    let bytes = sanitized.as_bytes();
    let mut end = 0;
    while end < bytes.len() && end < 2 && is_letter(bytes[end]) {
        end += 1;
    }
    Some(sanitized[..end].to_string())
}

/// Return the postcode sector: outcode, a single space, then the first incode digit.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_sector;
///
/// assert_eq!(to_sector("L27 8XY").as_deref(), Some("L27 8"));
/// ```
#[must_use]
pub fn to_sector(postcode: &str) -> Option<String> {
    let outcode = to_outcode(postcode)?;
    let incode = to_incode(postcode)?;
    let first = &incode[..1];
    Some(format!("{outcode} {first}"))
}

/// Return the postcode unit, the final two letters.
///
/// Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_unit;
///
/// assert_eq!(to_unit("L27 8XY").as_deref(), Some("XY"));
/// ```
#[must_use]
pub fn to_unit(postcode: &str) -> Option<String> {
    if !is_valid(postcode) {
        return None;
    }
    let sanitized = sanitize(postcode);
    let len = sanitized.len();
    Some(sanitized[len - 2..].to_string())
}

/// Return the postcode district.
///
/// When the outcode ends in a letter, the trailing letter is dropped. Otherwise
/// the whole outcode is returned. Returns `None` for an invalid postcode.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_district;
///
/// assert_eq!(to_district("AA9A 9AA").as_deref(), Some("AA9"));
/// assert_eq!(to_district("A99 9AA").as_deref(), Some("A99"));
/// ```
#[must_use]
pub fn to_district(postcode: &str) -> Option<String> {
    let outcode = to_outcode(postcode)?;
    match district_split(&outcode) {
        Some(district) => Some(district.to_string()),
        None => Some(outcode),
    }
}

/// Return the postcode sub-district.
///
/// Present only when the outcode ends in a letter, in which case the whole
/// outcode is the sub-district. Returns `None` when there is no sub-district or
/// the postcode is invalid.
///
/// # Examples
///
/// ```
/// use uk_postcode::to_sub_district;
///
/// assert_eq!(to_sub_district("AA9A 9AA").as_deref(), Some("AA9A"));
/// assert_eq!(to_sub_district("A9 9AA"), None);
/// ```
#[must_use]
pub fn to_sub_district(postcode: &str) -> Option<String> {
    let outcode = to_outcode(postcode)?;
    district_split(&outcode)?;
    Some(outcode)
}

/// Parse a postcode into all of its components at once.
///
/// Returns [`Postcode::Invalid`] for an invalid input. For a valid input returns
/// [`Postcode::Valid`] holding every component. The `sub_district` field is `Some`
/// only when the outcode ends in a letter.
///
/// # Examples
///
/// ```
/// use uk_postcode::parse;
///
/// let p = parse("Sw1A 2aa");
/// let valid = p.valid().expect("valid postcode");
/// assert_eq!(valid.postcode, "SW1A 2AA");
/// assert_eq!(valid.sector, "SW1A 2");
///
/// assert!(!parse("foo").is_valid());
/// ```
pub fn parse(postcode: &str) -> Postcode {
    let (Some(outcode), Some(incode)) = (to_outcode(postcode), to_incode(postcode)) else {
        return Postcode::Invalid;
    };
    let sector = format!("{outcode} {}", &incode[..1]);
    let unit = incode[1..].to_string();
    let (district, sub_district) = match district_split(&outcode) {
        Some(district) => (district.to_string(), Some(outcode.clone())),
        None => (outcode.clone(), None),
    };
    let area_end = outcode
        .bytes()
        .take(2)
        .take_while(|&b| is_letter(b))
        .count();
    Postcode::Valid(ValidPostcode {
        postcode: format!("{outcode} {incode}"),
        incode,
        area: outcode[..area_end].to_string(),
        district,
        sub_district,
        sector,
        unit,
        outcode,
    })
}

// --- corpus scanning -------------------------------------------------------

/// Find the end byte index of a postcode that starts at `start` in `bytes`.
///
/// Matches `POSTCODE_CORPUS_REGEX`: `[a-z]{1,2}\d[a-z\d]?\s*\d[a-z]{2}` without
/// anchors. Returns the index one past the match, or `None`.
///
/// The optional fourth outcode character is greedy, so the longer outcode is
/// tried first. If the incode cannot follow, the shorter outcode is tried. This
/// backtracking handles inputs like `RG45AY`.
fn corpus_match_end(s: &str, start: usize) -> Option<usize> {
    let bytes = s.as_bytes();
    let (min, max) = outcode_span(&bytes[start..])?;
    for out in [max, min] {
        let mut i = start + out;
        i += space_len(&s[i..]);
        let rest = &bytes[i..];
        if rest.len() >= 3 && is_incode(&rest[..3]) {
            return Some(i + 3);
        }
    }
    None
}

/// Iterate over every postcode in `corpus`, leftmost first.
///
/// Yields byte ranges. After a match the scan resumes at the match end, the same
/// as a global JavaScript regex. The optional fourth outcode character is greedy,
/// so a trailing letter is taken into the outcode when it fits.
fn corpus_matches(corpus: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let bytes = corpus.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match corpus_match_end(corpus, i) {
            Some(end) => {
                ranges.push((i, end));
                i = end;
            }
            None => i += 1,
        }
    }
    ranges
}

/// Find every postcode embedded in free text.
///
/// Returns the matches in document order with original casing and spacing. Bare
/// outward codes are not matched because they have no inward code. Returns an
/// empty vector when nothing matches.
///
/// # Examples
///
/// ```
/// use uk_postcode::match_corpus;
///
/// let found = match_corpus("PM lives at SW1A2Aa and SW1a 2AB");
/// assert_eq!(found, vec!["SW1A2Aa", "SW1a 2AB"]);
///
/// assert!(match_corpus("SW1 NW1 E1 E2").is_empty());
/// ```
#[must_use]
pub fn match_corpus(corpus: &str) -> Vec<String> {
    corpus_matches(corpus)
        .into_iter()
        .map(|(s, e)| corpus[s..e].to_string())
        .collect()
}

/// Replace every postcode in `corpus` with `replace_with`.
///
/// Returns the matches and the rewritten text. The replacement is literal, so
/// `$` characters carry no special meaning. Use `""` to delete the postcodes.
///
/// # Examples
///
/// ```
/// use uk_postcode::replace;
///
/// let out = replace("The PM lives at SW1A 2AA", "Downing Street");
/// assert_eq!(out.matches, vec!["SW1A 2AA"]);
/// assert_eq!(out.result, "The PM lives at Downing Street");
/// ```
pub fn replace(corpus: &str, replace_with: &str) -> ReplaceResult {
    let ranges = corpus_matches(corpus);
    let mut matches = Vec::with_capacity(ranges.len());
    let mut result = String::with_capacity(corpus.len());
    let mut last = 0;
    for (start, end) in ranges {
        matches.push(corpus[start..end].to_string());
        result.push_str(&corpus[last..start]);
        result.push_str(replace_with);
        last = end;
    }
    result.push_str(&corpus[last..]);
    ReplaceResult { matches, result }
}

// --- fix -------------------------------------------------------------------

/// Map a digit to the letter it is often confused with, for letter slots.
fn to_letter(c: u8) -> u8 {
    match c {
        b'0' => b'O',
        b'1' => b'I',
        other => other,
    }
}

/// Map a letter to the digit it is often confused with, for number slots.
fn to_number(c: u8) -> u8 {
    match c {
        b'O' => b'0',
        b'I' => b'1',
        other => other,
    }
}

/// Coerce `input` against a slot pattern.
///
/// Each character is read against the pattern character at the same index. `N`
/// forces a number, `L` forces a letter, `?` leaves the character alone. A
/// character past the end of the pattern is dropped.
fn coerce(pattern: &str, input: &str) -> String {
    let pat = pattern.as_bytes();
    let mut out = Vec::with_capacity(input.len());
    for (i, &c) in input.as_bytes().iter().enumerate() {
        match pat.get(i) {
            Some(b'N') => out.push(to_number(c)),
            Some(b'L') => out.push(to_letter(c)),
            Some(b'?') => out.push(c),
            _ => {}
        }
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Coerce an outward code by its length.
///
/// Length 2 uses `LN`, length 3 uses `L??`, length 4 uses `LLN?`. Any other
/// length is returned unchanged.
fn coerce_outcode(outcode: &str) -> String {
    match outcode.len() {
        2 => coerce("LN", outcode),
        3 => coerce("L??", outcode),
        4 => coerce("LLN?", outcode),
        _ => outcode.to_string(),
    }
}

/// Match `FIXABLE_REGEX`: `^\s*[a-z01]{1,2}[0-9oi][a-z\d]?\s*[0-9oi][a-z01]{2}\s*$`.
fn matches_fixable(s: &str) -> bool {
    // Class members. `[a-z01]` and `[0-9oi]` use the `i` flag, so letters fold case.
    fn is_letter_or_01(b: u8) -> bool {
        is_letter(b) || b == b'0' || b == b'1'
    }
    fn is_digit_oi(b: u8) -> bool {
        is_digit(b) || matches!(b, b'o' | b'O' | b'i' | b'I')
    }

    // Match the tail `[0-9oi][a-z\d]?\s*[0-9oi][a-z01]{2}\s*$` from `start`.
    //
    // The leading `[a-z01]{1,2}` is greedy and may take one or two characters.
    // Taking two can starve the required `[0-9oi]` that follows, for example
    // `01 OAA` needs the count to drop to one. This helper retries the tail with
    // the `[a-z\d]?` present then absent, which gives the needed backtracking.
    fn matches_tail(s: &str, start: usize) -> bool {
        let bytes = s.as_bytes();
        let n = bytes.len();
        let mut i = start;

        // [0-9oi]
        if i < n && is_digit_oi(bytes[i]) {
            i += 1;
        } else {
            return false;
        }

        // [a-z\d]? then the rest. Try with the optional character, then without.
        for take_optional in [true, false] {
            let mut j = i;
            if take_optional {
                if j >= n || !is_letter_or_digit(bytes[j]) {
                    continue;
                }
                j += 1;
            }
            // \s*
            j += space_len(&s[j..]);
            // [0-9oi]
            if j >= n || !is_digit_oi(bytes[j]) {
                continue;
            }
            j += 1;
            // [a-z01]{2}
            let mut ok = true;
            for _ in 0..2 {
                if j < n && is_letter_or_01(bytes[j]) {
                    j += 1;
                } else {
                    ok = false;
                    break;
                }
            }
            if !ok {
                continue;
            }
            // \s*$
            j += space_len(&s[j..]);
            if j == n {
                return true;
            }
        }
        false
    }

    let bytes = s.as_bytes();
    let n = bytes.len();
    let start = space_len(s);

    // [a-z01]{1,2}. Greedy, so try two characters before one.
    if start >= n || !is_letter_or_01(bytes[start]) {
        return false;
    }
    let can_take_two = start + 1 < n && is_letter_or_01(bytes[start + 1]);
    if can_take_two && matches_tail(s, start + 2) {
        return true;
    }
    matches_tail(s, start + 1)
}

/// Clean up a postcode, coercing commonly confused characters.
///
/// Swaps `O` and `0` and swaps `I` and `1` by position, trims, removes inner
/// whitespace, and inserts a single space between the outward and inward codes.
/// If the input cannot be coerced into shape, the original string is returned
/// unchanged, including its original casing and whitespace.
///
/// # Examples
///
/// ```
/// use uk_postcode::fix;
///
/// assert_eq!(fix(" Sw1A2aa "), "SW1A 2AA");
/// assert_eq!(fix("SW1A 2A0"), "SW1A 2AO");
/// assert_eq!(fix("hello"), "hello");
/// ```
#[must_use]
pub fn fix(s: &str) -> String {
    if !matches_fixable(s) {
        return s.to_string();
    }
    let cleaned = sanitize(s);
    let len = cleaned.len();
    let (outward, inward) = cleaned.split_at(len - 3);
    format!("{} {}", coerce_outcode(outward), coerce("NLL", inward))
}
