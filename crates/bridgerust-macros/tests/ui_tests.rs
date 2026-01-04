//! UI tests for macro error messages
//!
//! These tests use trybuild to verify compile-time errors are helpful

#[test]
fn ui_tests() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
