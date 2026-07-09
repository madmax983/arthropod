use trybuild::TestCases;

#[test]
fn test_macro_errors() {
    let t = TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
