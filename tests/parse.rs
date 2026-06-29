//! Coverage of the parsed `Postcode` components plus `is_valid_outcode`,
//! `match_corpus`, and `replace`.

mod common;

use common::load;
use uk_postcode::{
    is_valid_outcode, match_corpus, parse, replace, to_normalised, ParsePostcodeError, Postcode,
};

const CORPUS: &str = "SW1A2Aa is the residence of the Prime Minister. SW1a 2AB is the residence of her no.2. SW1A   1AA is where the queen lives. They are located in the SW1A outcode";

#[test]
fn parse_invalid_shape() {
    let p = parse("foo");
    assert_eq!(p, Postcode::Invalid);
    assert!(!p.is_valid());
    assert!(p.valid().is_none());
}

#[test]
fn from_str_parses_valid_and_rejects_invalid() {
    let p: Postcode = "Sw1A 2aa".parse().expect("valid postcode");
    assert_eq!(
        p.valid().map(|v| v.postcode.clone()),
        Some("SW1A 2AA".into())
    );

    let err = "foo".parse::<Postcode>().unwrap_err();
    assert_eq!(err, ParsePostcodeError);
}

#[test]
fn display_round_trips_through_from_str() {
    for input in ["SW1A 2AA", "L278XY", "ec1a 1bb"] {
        let p: Postcode = input.parse().expect("valid postcode");
        let text = p.to_string();
        let again: Postcode = text.parse().expect("normalised round-trips");
        assert_eq!(p, again, "round-trip {input:?}");
        assert_eq!(text, p.valid().unwrap().postcode);
    }
}

#[test]
fn parse_valid_flag_over_fixture() {
    for case in load("validation.json").tests {
        assert_eq!(
            parse(&case.base).is_valid(),
            case.expected_bool(),
            "parse({:?}).is_valid()",
            case.base
        );
    }
}

#[test]
fn parse_postcode_field_over_fixture() {
    for case in load("normalisation.json").tests {
        let got = parse(&case.base).valid().map(|v| v.postcode.clone());
        assert_eq!(got, case.expected_opt_str(), "parse({:?})", case.base);
    }
    assert!(parse("Definitly bogus").valid().is_none());
}

#[test]
fn parse_incode_field_over_fixture() {
    for case in load("incodes.json").tests {
        let got = parse(&case.base).valid().map(|v| v.incode.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitly bogus").valid().is_none());
}

#[test]
fn parse_outcode_field_over_fixture() {
    for case in load("outcodes.json").tests {
        let got = parse(&case.base).valid().map(|v| v.outcode.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitly bogus").valid().is_none());
}

#[test]
fn parse_area_field_over_fixture() {
    for case in load("areas.json").tests {
        let got = parse(&case.base).valid().map(|v| v.area.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitely bogus").valid().is_none());
}

#[test]
fn parse_district_field_over_fixture() {
    for case in load("districts.json").tests {
        let got = parse(&case.base).valid().map(|v| v.district.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitely bogus").valid().is_none());
}

#[test]
fn parse_sub_district_field_over_fixture() {
    for case in load("sub-districts.json").tests {
        let got = parse(&case.base)
            .valid()
            .and_then(|v| v.sub_district.clone());
        assert_eq!(got, case.expected_opt_str());
    }
    assert!(parse("Definitely bogus").valid().is_none());
}

#[test]
fn parse_sector_field_over_fixture() {
    for case in load("sectors.json").tests {
        let got = parse(&case.base).valid().map(|v| v.sector.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitely bogus").valid().is_none());
}

#[test]
fn parse_unit_field_over_fixture() {
    for case in load("units.json").tests {
        let got = parse(&case.base).valid().map(|v| v.unit.clone());
        assert_eq!(got, Some(case.expected_str()));
    }
    assert!(parse("Definitely bogus").valid().is_none());
}

#[test]
fn is_valid_outcode_true_for_every_fixture_outcode() {
    for case in load("outcodes.json").tests {
        let outcode = case.expected_str();
        assert!(is_valid_outcode(&outcode), "is_valid_outcode({outcode:?})");
    }
}

#[test]
fn is_valid_outcode_false_for_garbage() {
    for code in ["BOGUS", "Hello there", "12345"] {
        assert!(!is_valid_outcode(code), "is_valid_outcode({code:?})");
    }
}

#[test]
fn match_returns_matching_postcodes() {
    let found = match_corpus(CORPUS);
    assert_eq!(found, vec!["SW1A2Aa", "SW1a 2AB", "SW1A   1AA"]);
    let normalised: Vec<_> = found.iter().map(|s| to_normalised(s)).collect();
    assert_eq!(
        normalised,
        vec![
            Some("SW1A 2AA".to_string()),
            Some("SW1A 2AB".to_string()),
            Some("SW1A 1AA".to_string()),
        ]
    );
}

#[test]
fn match_returns_empty_for_outcodes_only() {
    assert!(match_corpus("SW1 NW1 E1 E2").is_empty());
}

#[test]
fn replace_default_removes_postcodes() {
    let out = replace(CORPUS, "");
    assert_eq!(out.matches, vec!["SW1A2Aa", "SW1a 2AB", "SW1A   1AA"]);
    assert_eq!(
        out.result,
        " is the residence of the Prime Minister.  is the residence of her no.2.  is where the queen lives. They are located in the SW1A outcode"
    );
}

#[test]
fn replace_with_custom_string() {
    let out = replace(CORPUS, "POSTCODE");
    assert_eq!(out.matches, vec!["SW1A2Aa", "SW1a 2AB", "SW1A   1AA"]);
    assert_eq!(
        out.result,
        "POSTCODE is the residence of the Prime Minister. POSTCODE is the residence of her no.2. POSTCODE is where the queen lives. They are located in the SW1A outcode"
    );
}

#[test]
fn replace_no_match_leaves_corpus_unchanged() {
    let out = replace("SW1 NW1 E1 E2", "");
    assert!(out.matches.is_empty());
    assert_eq!(out.result, "SW1 NW1 E1 E2");
}
