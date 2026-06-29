//! Coverage of the `parse` struct fields plus `valid_outcode`, `match_corpus`,
//! and `replace`.

mod common;

use common::load;
use uk_postcode::{match_corpus, parse, replace, to_normalised, valid_outcode, Postcode};

const CORPUS: &str = "SW1A2Aa is the residence of the Prime Minister. SW1a 2AB is the residence of her no.2. SW1A   1AA is where the queen lives. They are located in the SW1A outcode";

#[test]
fn parse_invalid_shape() {
    let p = parse("foo");
    assert_eq!(p, Postcode::default());
    assert!(!p.valid);
    assert_eq!(p.postcode, None);
    assert_eq!(p.incode, None);
    assert_eq!(p.outcode, None);
    assert_eq!(p.area, None);
    assert_eq!(p.district, None);
    assert_eq!(p.sub_district, None);
    assert_eq!(p.sector, None);
    assert_eq!(p.unit, None);
}

#[test]
fn parse_valid_flag_over_fixture() {
    for case in load("validation.json").tests {
        assert_eq!(
            parse(&case.base).valid,
            case.expected_bool(),
            "parse({:?}).valid",
            case.base
        );
    }
}

#[test]
fn parse_postcode_field_over_fixture() {
    for case in load("normalisation.json").tests {
        assert_eq!(
            parse(&case.base).postcode,
            case.expected_opt_str(),
            "parse({:?}).postcode",
            case.base
        );
    }
    assert_eq!(parse("Definitly bogus").postcode, None);
}

#[test]
fn parse_incode_field_over_fixture() {
    for case in load("incodes.json").tests {
        assert_eq!(parse(&case.base).incode, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitly bogus").incode, None);
}

#[test]
fn parse_outcode_field_over_fixture() {
    for case in load("outcodes.json").tests {
        assert_eq!(parse(&case.base).outcode, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitly bogus").outcode, None);
}

#[test]
fn parse_area_field_over_fixture() {
    for case in load("areas.json").tests {
        assert_eq!(parse(&case.base).area, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitely bogus").area, None);
}

#[test]
fn parse_district_field_over_fixture() {
    for case in load("districts.json").tests {
        assert_eq!(parse(&case.base).district, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitely bogus").district, None);
}

#[test]
fn parse_sub_district_field_over_fixture() {
    for case in load("sub-districts.json").tests {
        assert_eq!(parse(&case.base).sub_district, case.expected_opt_str());
    }
    assert_eq!(parse("Definitely bogus").sub_district, None);
}

#[test]
fn parse_sector_field_over_fixture() {
    for case in load("sectors.json").tests {
        assert_eq!(parse(&case.base).sector, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitely bogus").sector, None);
}

#[test]
fn parse_unit_field_over_fixture() {
    for case in load("units.json").tests {
        assert_eq!(parse(&case.base).unit, Some(case.expected_str()));
    }
    assert_eq!(parse("Definitely bogus").unit, None);
}

#[test]
fn valid_outcode_true_for_every_fixture_outcode() {
    for case in load("outcodes.json").tests {
        let outcode = case.expected_str();
        assert!(valid_outcode(&outcode), "valid_outcode({outcode:?})");
    }
}

#[test]
fn valid_outcode_false_for_garbage() {
    for code in ["BOGUS", "Hello there", "12345"] {
        assert!(!valid_outcode(code), "valid_outcode({code:?})");
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
