use crate::{Adapter, UnstructuredFullReadFacts};
use docnav_protocol::{Operation, OutlineArguments, RequestEnvelope};

use super::support::NoHookAdapter;

#[test]
fn unstructured_full_read_hooks_default_to_absent_capabilities() {
    let adapter = NoHookAdapter;
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

    let capabilities = adapter.unstructured_full_read_capabilities();

    assert!(!capabilities.content_hook);
    assert!(!capabilities.result_facts_hook);
    assert!(capabilities.cost_measurement_units.is_empty());
    assert!(!capabilities.has_cost_measurement_unit("tokens"));
    assert!(adapter.unstructured_full_read(&request).is_err());
    assert_eq!(
        adapter
            .measure_unstructured_full_read_cost(&request, &["tokens".to_owned()])
            .unwrap()
            .measurements,
        Vec::new()
    );
    assert_eq!(
        adapter.unstructured_full_read_facts(&request).unwrap(),
        UnstructuredFullReadFacts::default()
    );
}
