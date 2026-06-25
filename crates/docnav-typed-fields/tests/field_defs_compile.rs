#[test]
fn field_defs_reject_invalid_declarations_at_compile_time() {
    let tests = trybuild::TestCases::new();
    tests.compile_fail("tests/ui/field_defs_type_mismatch.rs");
    tests.compile_fail("tests/ui/field_defs_missing_validation.rs");
    tests.compile_fail("tests/ui/field_defs_missing_field_attr.rs");
}
