#[test]
fn symbolix_compile_fail_cases() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
