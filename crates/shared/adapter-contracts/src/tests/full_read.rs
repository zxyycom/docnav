use crate::{AdapterDefinition, UnstructuredFullReadFacts};
use docnav_protocol::{Operation, OutlineArguments, RequestEnvelope};

use super::support::{no_hook_manifest, NoHookAdapter};

#[test]
fn unstructured_full_read_hooks_default_to_absent_capabilities() {
    let adapter = NoHookAdapter;
    let definition =
        AdapterDefinition::new(no_hook_manifest(), &adapter, None).expect("valid definition");
    let request = RequestEnvelope {
        protocol_version: docnav_protocol::PROTOCOL_VERSION.to_owned(),
        request_id: "req-hooks".to_owned(),
        operation: Operation::Outline,
        document: docnav_protocol::Document {
            path: "doc.stub".to_owned(),
        },
        arguments: docnav_protocol::OperationArguments::Outline(OutlineArguments {
            limit: docnav_protocol::positive_result(80).unwrap(),
            page: docnav_protocol::positive_result(1).unwrap(),
            options: None,
        }),
    };

    assert!(definition.unstructured_full_read_capabilities().is_none());
    assert!(definition.unstructured_full_read(&request).is_err());
    assert_eq!(
        definition
            .measure_unstructured_full_read_cost(&request, &["tokens".to_owned()])
            .unwrap()
            .measurements,
        Vec::new()
    );
    assert_eq!(
        definition.unstructured_full_read_facts(&request).unwrap(),
        UnstructuredFullReadFacts::default()
    );
}
