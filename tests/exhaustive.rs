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
        assert!(parse(&p).is_valid(), "parse({p:?}).is_valid()");
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
        // Drop the first whitespace run, the same as a single-replace of the space.
        let unspaced = p.replacen(' ', "", 1);
        let stripped = parse(&unspaced);

        let base = base.valid().expect("base valid");
        let lower = lower.valid().expect("lower valid");
        let stripped = stripped.valid().expect("stripped valid");

        // Casing and spacing do not change any component.
        assert_eq!(base, lower, "lower {p:?}");
        assert_eq!(base, stripped, "stripped {p:?}");
    }
}
