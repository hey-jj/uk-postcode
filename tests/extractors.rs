//! Coverage of the validator and component extractor functions, driven by the
//! JSON fixtures. Covers `is_valid`, `to_normalised`, `to_outcode`, `to_incode`,
//! `to_area`, `to_district`, `to_sub_district`, `to_sector`, and `to_unit`.

mod common;

use common::load;
use uk_postcode::{
    is_valid, to_area, to_district, to_incode, to_normalised, to_outcode, to_sector,
    to_sub_district, to_unit,
};

#[test]
fn is_valid_over_validation_fixture() {
    for case in load("validation.json").tests {
        assert_eq!(
            is_valid(&case.base),
            case.expected_bool(),
            "is_valid({:?})",
            case.base
        );
    }
}

#[test]
fn to_normalised_over_fixture() {
    for case in load("normalisation.json").tests {
        assert_eq!(
            to_normalised(&case.base),
            case.expected_opt_str(),
            "to_normalised({:?})",
            case.base
        );
    }
    assert_eq!(to_normalised("Definitly bogus"), None);
}

#[test]
fn to_incode_over_fixture() {
    for case in load("incodes.json").tests {
        assert_eq!(
            to_incode(&case.base),
            Some(case.expected_str()),
            "to_incode({:?})",
            case.base
        );
    }
    assert_eq!(to_incode("Definitly bogus"), None);
}

#[test]
fn to_outcode_over_fixture() {
    for case in load("outcodes.json").tests {
        assert_eq!(
            to_outcode(&case.base),
            Some(case.expected_str()),
            "to_outcode({:?})",
            case.base
        );
    }
    assert_eq!(to_outcode("Definitly bogus"), None);
}

#[test]
fn to_area_over_fixture() {
    for case in load("areas.json").tests {
        assert_eq!(
            to_area(&case.base),
            Some(case.expected_str()),
            "to_area({:?})",
            case.base
        );
    }
    assert_eq!(to_area("Definitely bogus"), None);
}

#[test]
fn to_district_over_fixture() {
    for case in load("districts.json").tests {
        assert_eq!(
            to_district(&case.base),
            Some(case.expected_str()),
            "to_district({:?})",
            case.base
        );
    }
    assert_eq!(to_district("Definitely bogus"), None);
}

#[test]
fn to_sub_district_over_fixture() {
    for case in load("sub-districts.json").tests {
        assert_eq!(
            to_sub_district(&case.base),
            case.expected_opt_str(),
            "to_sub_district({:?})",
            case.base
        );
    }
    assert_eq!(to_sub_district("Definitely bogus"), None);
}

#[test]
fn to_sector_over_fixture() {
    for case in load("sectors.json").tests {
        assert_eq!(
            to_sector(&case.base),
            Some(case.expected_str()),
            "to_sector({:?})",
            case.base
        );
    }
    assert_eq!(to_sector("Definitely bogus"), None);
}

#[test]
fn to_unit_over_fixture() {
    for case in load("units.json").tests {
        assert_eq!(
            to_unit(&case.base),
            Some(case.expected_str()),
            "to_unit({:?})",
            case.base
        );
    }
    assert_eq!(to_unit("Definitely bogus"), None);
}
