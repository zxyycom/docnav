use docnav_protocol::{
    AdapterIdentity, FormatDescriptor, Manifest, Operation, ProbeResult, PROTOCOL_VERSION,
};

use super::*;

struct BareAdapter;

impl Adapter for BareAdapter {
    fn adapter_id(&self) -> &str {
        "bare"
    }

    fn manifest(&self) -> Manifest {
        Manifest {
            manifest_version: PROTOCOL_VERSION.to_owned(),
            adapter: AdapterIdentity {
                id: "bare".to_owned(),
                name: "Bare".to_owned(),
                version: "0.1.0".to_owned(),
            },
            formats: vec![FormatDescriptor {
                id: "bare".to_owned(),
                extensions: vec![".bare".to_owned()],
                content_types: vec!["text/bare".to_owned()],
            }],
            capabilities: Vec::new(),
        }
    }

    fn probe(&self, _path: &str) -> ProbeResult {
        unreachable!("contract error test does not probe")
    }
}

#[test]
fn default_unsupported_operation_maps_to_protocol_error() {
    let error = BareAdapter.unsupported(Operation::Read);
    let protocol_error = error.protocol_error();

    assert_eq!(
        protocol_error.code().protocol_code(),
        "CAPABILITY_UNSUPPORTED"
    );
    assert_eq!(protocol_error.owner(), "adapter");
    assert_eq!(
        protocol_error
            .details()
            .get("adapter_id")
            .and_then(|value| value.as_str()),
        Some("bare")
    );
    assert_eq!(
        protocol_error
            .details()
            .get("capability")
            .and_then(|value| value.as_str()),
        Some("read")
    );
}

#[test]
fn native_option_specs_keep_same_key_owner_and_type_variants() {
    let integer_variant = NativeOptionSpec {
        identity: "docnav.adapters.integer.options.shared",
        owner: "integer-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: NativeOptionValueSpec::Integer { min: 1, max: 3 },
    };
    let string_variant = NativeOptionSpec {
        identity: "docnav.adapters.string.options.shared",
        owner: "string-adapter",
        namespace: "options",
        key: "shared",
        operations: &[Operation::Outline],
        value: NativeOptionValueSpec::String,
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
