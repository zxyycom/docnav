use std::num::NonZeroU32;

use docnav_protocol::Operation;
use docnav_typed_fields::ValueKind;

use crate::{
    AdapterDefinition, FindInput, InfoInput, OutlineInput, ReadInput, StandardInputBinding,
    StandardOperationInput,
};

use super::support::{no_hook_manifest, NoHookAdapter};

#[test]
fn standard_operation_input_exposes_closed_operation_specific_values() {
    for input in standard_inputs() {
        let operation = input.operation();
        match input {
            StandardOperationInput::Outline(input) => {
                assert_eq!(operation, Operation::Outline);
                assert_eq!(input.document_path, "outline.md");
                assert_eq!((input.page.get(), input.limit.get()), (1, 10));
                assert_eq!(input.max_heading_level, Some(3));
            }
            StandardOperationInput::Read(input) => {
                assert_eq!(operation, Operation::Read);
                assert_eq!(input.document_path, "read.md");
                assert_eq!(input.ref_id, "H:L1:H1");
                assert_eq!((input.page.get(), input.limit.get()), (2, 20));
            }
            StandardOperationInput::Find(input) => {
                assert_eq!(operation, Operation::Find);
                assert_eq!(input.document_path, "find.md");
                assert_eq!(input.query, "needle");
                assert_eq!((input.page.get(), input.limit.get()), (3, 30));
                assert_eq!(input.max_heading_level, None);
            }
            StandardOperationInput::Info(input) => {
                assert_eq!(operation, Operation::Info);
                assert_eq!(input.document_path, "info.md");
            }
        }
    }
}

#[test]
fn adapter_definition_dispatches_closed_standard_input_variants() {
    let adapter = NoHookAdapter;
    let definition =
        AdapterDefinition::new(no_hook_manifest(), &adapter, None).expect("valid definition");

    for input in standard_inputs() {
        let operation = input.operation();
        let result = definition
            .execute_operation(&input)
            .expect("standard input dispatch");

        assert_eq!(result.operation(), operation);
    }
}

#[test]
fn standard_input_bindings_report_operation_and_expected_value_kind() {
    let bindings = [
        StandardInputBinding::OutlinePage,
        StandardInputBinding::OutlineLimit,
        StandardInputBinding::OutlineMaxHeadingLevel,
        StandardInputBinding::ReadPage,
        StandardInputBinding::ReadLimit,
        StandardInputBinding::FindPage,
        StandardInputBinding::FindLimit,
        StandardInputBinding::FindMaxHeadingLevel,
    ];

    let observed = bindings.map(|binding| (binding.operation(), binding.expected_value_kind()));

    assert_eq!(
        observed,
        [
            (Operation::Outline, ValueKind::Integer),
            (Operation::Outline, ValueKind::Integer),
            (Operation::Outline, ValueKind::Integer),
            (Operation::Read, ValueKind::Integer),
            (Operation::Read, ValueKind::Integer),
            (Operation::Find, ValueKind::Integer),
            (Operation::Find, ValueKind::Integer),
            (Operation::Find, ValueKind::Integer),
        ]
    );
}

fn positive(value: u32) -> NonZeroU32 {
    NonZeroU32::new(value).expect("fixture values are positive")
}

fn standard_inputs() -> [StandardOperationInput; 4] {
    [
        StandardOperationInput::Outline(OutlineInput {
            document_path: "outline.md".to_owned(),
            page: positive(1),
            limit: positive(10),
            max_heading_level: Some(3),
        }),
        StandardOperationInput::Read(ReadInput {
            document_path: "read.md".to_owned(),
            ref_id: "H:L1:H1".to_owned(),
            page: positive(2),
            limit: positive(20),
        }),
        StandardOperationInput::Find(FindInput {
            document_path: "find.md".to_owned(),
            query: "needle".to_owned(),
            page: positive(3),
            limit: positive(30),
            max_heading_level: None,
        }),
        StandardOperationInput::Info(InfoInput {
            document_path: "info.md".to_owned(),
        }),
    ]
}
