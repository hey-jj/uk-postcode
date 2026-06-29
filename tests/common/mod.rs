//! Shared fixture loader for the conformance suite.
//!
//! Each fixture file holds `{ "tests": [ { "base": ..., "expected": ... } ] }`.
//! The `expected` value is a string, a bool, or null depending on the file, so
//! it loads as a `serde_json::Value` and each test reads it in the shape it needs.

use serde::Deserialize;
use std::path::PathBuf;

/// One fixture file: a list of cases.
#[derive(Deserialize)]
pub struct Fixtures {
    /// The cases in the file.
    pub tests: Vec<Case>,
}

/// One fixture case.
#[derive(Deserialize)]
pub struct Case {
    /// Input passed to the function under test.
    pub base: String,
    /// Expected output. String, bool, or null.
    pub expected: serde_json::Value,
}

impl Case {
    /// The expected value as a string, or `None` for JSON null.
    pub fn expected_opt_str(&self) -> Option<String> {
        match &self.expected {
            serde_json::Value::Null => None,
            serde_json::Value::String(s) => Some(s.clone()),
            other => panic!("expected string or null, got {other:?}"),
        }
    }

    /// The expected value as a string. Panics on null.
    pub fn expected_str(&self) -> String {
        self.expected
            .as_str()
            .unwrap_or_else(|| panic!("expected string, got {:?}", self.expected))
            .to_string()
    }

    /// The expected value as a bool.
    pub fn expected_bool(&self) -> bool {
        self.expected
            .as_bool()
            .unwrap_or_else(|| panic!("expected bool, got {:?}", self.expected))
    }
}

/// Load a fixture file from `tests/fixtures` by name.
pub fn load(name: &str) -> Fixtures {
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("tests/fixtures");
    path.push(name);
    let text =
        std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    serde_json::from_str(&text).unwrap_or_else(|e| panic!("parse {}: {e}", path.display()))
}
