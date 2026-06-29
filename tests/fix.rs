//! Coercion behaviour of `fix`.

use uk_postcode::fix;

#[test]
fn fix_cases() {
    let cases: &[(&str, &str)] = &[
        // trim, case, spacing
        (" SW1A 2AA ", "SW1A 2AA"),
        (" Sw1A 2aa ", "SW1A 2AA"),
        (" Sw1A2aa ", "SW1A 2AA"),
        (" Sw1A  2aa ", "SW1A 2AA"),
        // not fixable, returns original unchanged
        (" 1A2aa ", " 1A2aa "),
        // outward LN format
        ("01 OAA", "O1 0AA"),
        ("SO OAA", "S0 0AA"),
        // outward L?? format
        ("0W1 OAA", "OW1 0AA"),
        ("S01 OAA", "S01 0AA"),
        ("SO1 OAA", "SO1 0AA"),
        ("SWO OAA", "SWO 0AA"),
        ("SW0 OAA", "SW0 0AA"),
        // outward LLN? format
        ("0W1A OAA", "OW1A 0AA"),
        ("S01A OAA", "SO1A 0AA"),
        ("SWOA OAA", "SW0A 0AA"),
        ("SW10 OAA", "SW10 0AA"),
        ("SW1O OAA", "SW1O 0AA"),
        // inward code
        (" SW1A OAA", "SW1A 0AA"),
        ("SW1A 20A", "SW1A 2OA"),
        ("SW1A 2A0", "SW1A 2AO"),
        // I <=> 1
        ("SWIA 2AA", "SW1A 2AA"),
        ("1W1A 2AA", "IW1A 2AA"),
    ];

    for (input, expected) in cases {
        assert_eq!(fix(input), *expected, "fix({input:?})");
    }
}
