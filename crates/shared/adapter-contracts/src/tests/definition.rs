use crate::{AdapterDefinition, AdapterDefinitionError, UnstructuredFullReadCapabilities};

use super::support::{no_hook_manifest, NoHookAdapter};

#[test]
fn adapter_definition_rejects_invalid_full_read_capabilities() {
    let adapter = NoHookAdapter;
    let cases = [
        UnstructuredFullReadCapabilities::default(),
        UnstructuredFullReadCapabilities {
            content_hook: false,
            cost_measurement_units: vec![String::new()],
            result_facts_hook: false,
        },
        UnstructuredFullReadCapabilities {
            content_hook: false,
            cost_measurement_units: vec!["tokens".to_owned(), "tokens".to_owned()],
            result_facts_hook: false,
        },
    ];

    for capabilities in cases {
        let error = AdapterDefinition::new(no_hook_manifest(), &adapter, Some(capabilities))
            .expect_err("invalid full-read capabilities");

        assert!(matches!(
            error,
            AdapterDefinitionError::UnsupportedCapabilityCombination {
                capability: "full_read",
                ..
            }
        ));
    }
}
