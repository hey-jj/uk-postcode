//! Tests for behaviour the fixtures imply but never assert directly.

use uk_postcode::{
    fix, is_valid, match_corpus, parse, replace, to_area, to_district, to_incode, to_normalised,
    to_outcode, to_sector, to_sub_district, to_unit,
};

#[test]
fn gir_0aa_is_invalid() {
    // The Girobank postcode does not fit the standard shape, so the library
    // rejects it.
    assert!(!is_valid("GIR 0AA"));
    assert!(!parse("GIR 0AA").is_valid());
    assert_eq!(to_outcode("GIR 0AA"), None);
}

#[test]
fn empty_and_whitespace_inputs() {
    assert!(!is_valid(""));
    assert!(!is_valid("   "));

    assert!(!parse("").is_valid());

    assert!(match_corpus("").is_empty());

    let out = replace("", "X");
    assert!(out.matches.is_empty());
    assert_eq!(out.result, "");

    assert_eq!(fix(""), "");
    assert_eq!(fix("   "), "   ");
}

#[test]
fn fix_unfixable_returns_original() {
    for input in ["hello", "12345", " 1A2aa ", "", "   ", "!!!"] {
        assert_eq!(fix(input), input, "fix({input:?})");
    }
}

#[test]
fn match_multiline_and_adjacent() {
    let corpus = "Two homes:\nSW1A 2AA\nEC1A 1BB end";
    assert_eq!(match_corpus(corpus), vec!["SW1A 2AA", "EC1A 1BB"]);

    let out = replace(corpus, "[redacted]");
    assert_eq!(out.result, "Two homes:\n[redacted]\n[redacted] end");

    // Back-to-back postcodes with a single separating space.
    let adjacent = "SW1A 2AA EC1A 1BB";
    assert_eq!(match_corpus(adjacent), vec!["SW1A 2AA", "EC1A 1BB"]);
}

#[test]
fn unicode_input_stays_invalid() {
    for input in ["SW1Aß 2AA", "С\u{0421}1A 2AA", "ＳＷ１Ａ ２ＡＡ"] {
        assert!(!is_valid(input), "is_valid({input:?})");
        assert_eq!(to_outcode(input), None);
        assert_eq!(to_incode(input), None);
        assert_eq!(to_area(input), None);
        assert_eq!(to_district(input), None);
        assert_eq!(to_sub_district(input), None);
        assert_eq!(to_sector(input), None);
        assert_eq!(to_unit(input), None);
        assert_eq!(to_normalised(input), None);
        assert!(!parse(input).is_valid());
        assert_eq!(fix(input), input, "fix({input:?})");
    }
}

#[test]
fn sub_district_split_boundary() {
    // Outcodes ending in a letter have a sub-district equal to the outcode and a
    // district that drops the trailing letter.
    let with_sub = [
        ("EC1A 1BB", "EC1A", "EC1"),
        ("W1A 0AX", "W1A", "W1"),
        ("NW1W 1AA", "NW1W", "NW1"),
        ("N1C 0AB", "N1C", "N1"),
        ("A9A 9AA", "A9A", "A9"),
        ("AA9A 9AA", "AA9A", "AA9"),
    ];
    for (pc, sub, district) in with_sub {
        assert_eq!(to_sub_district(pc).as_deref(), Some(sub), "sub {pc:?}");
        assert_eq!(
            to_district(pc).as_deref(),
            Some(district),
            "district {pc:?}"
        );
    }

    // Outcodes ending in a digit have no sub-district and a district equal to the
    // whole outcode.
    let without_sub = [
        ("A9 9AA", "A9"),
        ("A99 9AA", "A99"),
        ("AA9 9AA", "AA9"),
        ("AA99 9AA", "AA99"),
    ];
    for (pc, outcode) in without_sub {
        assert_eq!(to_sub_district(pc), None, "sub {pc:?}");
        assert_eq!(to_district(pc).as_deref(), Some(outcode), "district {pc:?}");
    }
}

#[test]
fn normalise_is_idempotent() {
    for input in [
        "L27 8XY",
        "L278XY",
        "NR10 3EZ",
        "NR103EZ",
        "sw1a2aa",
        "  not a postcode",
    ] {
        if let Some(once) = to_normalised(input) {
            assert_eq!(to_normalised(&once), Some(once.clone()), "{input:?}");
        }
    }
}

#[test]
fn parse_valid_fields_are_populated() {
    // Every valid postcode fills every component. The enum makes the always-present
    // fields non-optional, so this checks they hold the expected values.
    for pc in ["L27 8XY", "SW1A 2AA", "EC1A 1BB"] {
        let p = parse(pc);
        let v = p.valid().expect("valid postcode");
        assert!(!v.postcode.is_empty());
        assert!(!v.incode.is_empty());
        assert!(!v.outcode.is_empty());
        assert!(!v.area.is_empty());
        assert!(!v.district.is_empty());
        assert!(!v.sector.is_empty());
        assert!(!v.unit.is_empty());
    }
}

#[test]
fn non_ascii_whitespace_separates_a_valid_postcode() {
    // The shape allows whitespace between the outward and inward codes, and the
    // whitespace set covers the ECMAScript marks. A postcode split by one of these
    // stays valid and normalises to a single ASCII space.
    let separators = [
        '\u{00A0}', // no-break space
        '\u{2028}', // line separator
        '\u{2029}', // paragraph separator
        '\u{3000}', // ideographic space
    ];
    for sep in separators {
        let input = format!("SW1A{sep}2AA");
        assert!(is_valid(&input), "is_valid({input:?})");
        assert_eq!(
            to_normalised(&input).as_deref(),
            Some("SW1A 2AA"),
            "to_normalised({input:?})"
        );
        assert_eq!(to_outcode(&input).as_deref(), Some("SW1A"), "{input:?}");
        assert_eq!(to_incode(&input).as_deref(), Some("2AA"), "{input:?}");
    }
}

#[test]
fn whitespace_set_boundary_code_points() {
    // U+FEFF (byte order mark) counts as whitespace here even though Rust's own
    // char::is_whitespace excludes it. U+180E (mongolian vowel separator) is not
    // whitespace under either rule. These two pin the hand-written set.
    let bom = "SW1A\u{FEFF}2AA";
    assert!(is_valid(bom));
    assert_eq!(to_normalised(bom).as_deref(), Some("SW1A 2AA"));
    assert_eq!(to_outcode(bom).as_deref(), Some("SW1A"));
    assert_eq!(fix(bom), "SW1A 2AA");

    let mvs = "SW1A\u{180E}2AA";
    assert!(!is_valid(mvs));
    assert_eq!(to_normalised(mvs), None);
    assert_eq!(to_outcode(mvs), None);
}

#[test]
fn replace_treats_the_replacement_as_literal() {
    // The replacement string is inserted verbatim. Sequences that a regex replace
    // would expand carry no special meaning here.
    let cases = [
        ("$&", "at $& now"),
        ("$$", "at $$ now"),
        ("$1", "at $1 now"),
        ("[$`|$']", "at [$`|$'] now"),
    ];
    for (replacement, expected) in cases {
        let out = replace("at SW1A 2AA now", replacement);
        assert_eq!(out.result, expected, "replace with {replacement:?}");
    }
}

#[test]
fn corpus_match_backtracks_on_the_spaceless_outcode_split() {
    // RG45AY has no separating space, so the optional fourth outcode character is
    // ambiguous. The scanner must read the 5 into the incode, not the outcode.
    assert_eq!(match_corpus("home RG45AY end"), vec!["RG45AY"]);
    assert_eq!(match_corpus("L278XY NR103EZ"), vec!["L278XY", "NR103EZ"]);

    let out = replace("home RG45AY end", "X");
    assert_eq!(out.matches, vec!["RG45AY"]);
    assert_eq!(out.result, "home X end");
}
