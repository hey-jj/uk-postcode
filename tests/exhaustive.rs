//! Round-trip invariants over a checked-in sample of real postcodes.
//!
//! The intent matches a full-dataset sweep: every sample postcode is valid, `fix`
//! leaves a normalised postcode untouched, and parsing is insensitive to case and
//! spacing. The sample is small and committed so the suite stays offline and
//! deterministic.

use uk_postcode::{fix, parse};

fn samples() -> Vec<String> {
    include_str!("fixtures/sample_postcodes.txt")
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(String::from)
        .collect()
}

#[test]
fn every_sample_is_valid() {
    for p in samples() {
        assert!(parse(&p).valid, "parse({p:?}).valid");
    }
}

#[test]
fn fix_never_changes_a_normalised_postcode() {
    for p in samples() {
        assert_eq!(fix(&p), p, "fix({p:?})");
    }
}

#[test]
fn parse_is_case_and_space_insensitive() {
    for p in samples() {
        let base = parse(&p);
        let lower = parse(&p.to_lowercase());
        // Drop the first whitespace run, the same as the source's replace(/\s/, "").
        let unspaced = p.replacen(' ', "", 1);
        let stripped = parse(&unspaced);

        assert_eq!(base.postcode, lower.postcode, "postcode {p:?}");
        assert_eq!(base.postcode, stripped.postcode, "postcode {p:?}");
        assert_eq!(base.incode, lower.incode, "incode {p:?}");
        assert_eq!(base.incode, stripped.incode, "incode {p:?}");
        assert_eq!(base.outcode, lower.outcode, "outcode {p:?}");
        assert_eq!(base.outcode, stripped.outcode, "outcode {p:?}");
        assert_eq!(base.area, lower.area, "area {p:?}");
        assert_eq!(base.area, stripped.area, "area {p:?}");
        assert_eq!(base.district, lower.district, "district {p:?}");
        assert_eq!(base.district, stripped.district, "district {p:?}");
        assert_eq!(base.sub_district, lower.sub_district, "sub_district {p:?}");
        assert_eq!(
            base.sub_district, stripped.sub_district,
            "sub_district {p:?}"
        );
        assert_eq!(base.sector, lower.sector, "sector {p:?}");
        assert_eq!(base.sector, stripped.sector, "sector {p:?}");
        assert_eq!(base.unit, lower.unit, "unit {p:?}");
        assert_eq!(base.unit, stripped.unit, "unit {p:?}");
    }
}
