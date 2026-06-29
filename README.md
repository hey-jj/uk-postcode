# uk-postcode

Parse, validate, and normalize UK postcodes.

This crate checks the shape of a postcode. It does not look the postcode up
against a real list, so a string with a valid shape may still not name a real
place. A full check needs the Royal Mail or Ordnance Survey postcode data.

## Installation

```toml
[dependencies]
uk-postcode = "0.1"
```

## Usage

```rust
use uk_postcode::{is_valid, to_normalised, parse, fix, match_corpus, replace};

// Validate the shape.
assert!(is_valid("SW1A 2AA"));
assert!(!is_valid("Definitely wrong"));

// Normalise casing and spacing.
assert_eq!(to_normalised("sw1a2aa").as_deref(), Some("SW1A 2AA"));

// Pull every component at once.
let p = parse("Sw1A 2aa");
assert!(p.valid);
assert_eq!(p.outcode.as_deref(), Some("SW1A"));
assert_eq!(p.incode.as_deref(), Some("2AA"));
assert_eq!(p.area.as_deref(), Some("SW"));
assert_eq!(p.district.as_deref(), Some("SW1"));
assert_eq!(p.sub_district.as_deref(), Some("SW1A"));
assert_eq!(p.sector.as_deref(), Some("SW1A 2"));
assert_eq!(p.unit.as_deref(), Some("AA"));

// Coerce common typos like O for 0 and I for 1.
assert_eq!(fix("SW1A 2A0"), "SW1A 2AO");

// Find or replace postcodes in free text.
assert_eq!(match_corpus("PM lives at SW1A 2AA"), vec!["SW1A 2AA"]);
let out = replace("PM lives at SW1A 2AA", "[redacted]");
assert_eq!(out.result, "PM lives at [redacted]");
```

## API

| Function | Returns | Notes |
|---|---|---|
| `is_valid(&str)` | `bool` | Whole-string shape check on the raw input. |
| `valid_outcode(&str)` | `bool` | Whole-string outward-code check. |
| `to_normalised(&str)` | `Option<String>` | `"OUTCODE INCODE"`, uppercased. |
| `to_outcode(&str)` | `Option<String>` | Outward code, for example `SW1A`. |
| `to_incode(&str)` | `Option<String>` | Inward code, for example `2AA`. |
| `to_area(&str)` | `Option<String>` | Leading one or two letters. |
| `to_district(&str)` | `Option<String>` | Outcode without a trailing sub-district letter. |
| `to_sub_district(&str)` | `Option<String>` | Whole outcode when it ends in a letter, else `None`. |
| `to_sector(&str)` | `Option<String>` | Outcode plus the first incode digit. |
| `to_unit(&str)` | `Option<String>` | Final two letters. |
| `parse(&str)` | `Postcode` | Every component, or all `None` when invalid. |
| `match_corpus(&str)` | `Vec<String>` | Postcodes found in free text, original formatting. |
| `replace(&str, &str)` | `ReplaceResult` | Matches plus the text with each match replaced. |
| `fix(&str)` | `String` | Coerced and reformatted, or the input unchanged. |

Parsers return `None` for any input that is not a valid postcode shape.

## Postcode components

A postcode splits into an outward code and an inward code. Here `A` is a letter
and `9` is a digit.

| Schema     | outcode | incode | area | district | sub-district | sector   | unit |
|------------|---------|--------|------|----------|--------------|----------|------|
| `AA9A 9AA` | `AA9A`  | `9AA`  | `AA` | `AA9`    | `AA9A`       | `AA9A 9` | `AA` |
| `A9A 9AA`  | `A9A`   | `9AA`  | `A`  | `A9`     | `A9A`        | `A9A 9`  | `AA` |
| `A9 9AA`   | `A9`    | `9AA`  | `A`  | `A9`     | none         | `A9 9`   | `AA` |
| `A99 9AA`  | `A99`   | `9AA`  | `A`  | `A99`    | none         | `A99 9`  | `AA` |
| `AA9 9AA`  | `AA9`   | `9AA`  | `AA` | `AA9`    | none         | `AA9 9`  | `AA` |
| `AA99 9AA` | `AA99`  | `9AA`  | `AA` | `AA99`   | none         | `AA99 9` | `AA` |

## License

Licensed under the [MIT license](LICENSE).
</content>
