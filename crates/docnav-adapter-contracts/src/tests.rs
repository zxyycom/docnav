use super::*;
use docnav_protocol::Operation;

#[test]
fn native_option_specs_keep_same_key_owner_and_type_variants() {
    let integer_variant = NativeOptionSpec {
        identity: "docnav.adapters.integer.options.shared",
        owner: "integer-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        cli_flag: None,
        value: NativeOptionValueSpec::Integer { min: 1, max: 3 },
        default: None,
    };
    let string_variant = NativeOptionSpec {
        identity: "docnav.adapters.string.options.shared",
        owner: "string-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        cli_flag: None,
        value: NativeOptionValueSpec::String,
        default: None,
    };

    let specs = [integer_variant, string_variant];

    assert_eq!(specs[0].key, specs[1].key);
    assert_ne!(specs[0].identity, specs[1].identity);
    assert_ne!(specs[0].owner, specs[1].owner);
    assert_eq!(specs[0].value_kind(), NativeOptionValueKind::Integer);
    assert_eq!(specs[1].value_kind(), NativeOptionValueKind::String);
    assert_eq!(specs[0].namespace, specs[1].namespace);
    assert_ne!(specs[0].value_kind(), specs[1].value_kind());
}
