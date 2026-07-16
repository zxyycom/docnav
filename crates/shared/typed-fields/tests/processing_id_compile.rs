#[test]
fn processing_id_has_no_unchecked_from_conversion() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/processing_id_unchecked_from.rs");
}
