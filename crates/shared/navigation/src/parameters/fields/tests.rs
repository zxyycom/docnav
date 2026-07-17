use docnav_adapter_contracts::StandardInputBinding;
use docnav_protocol::Operation;
use docnav_typed_fields::{
    ExpectedFieldShape, FieldDef, FieldDefSet, FieldIdentity, FieldValidation, ProcessStrategy,
};

use super::operation_fields;
use crate::parameters::{
    ids, DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterEntry,
};

const MATCHING_IDENTITY: &str = "docnav.adapters.first.options.max_heading_level";
const OTHER_IDENTITY: &str = "docnav.adapters.second.options.max_heading_level";

// @case WB-NAVIGATION-FIELD-SETS-001
#[test]
fn selected_fields_combine_fixed_inputs_with_catalog_projection() {
    let catalog = catalog();

    let selected =
        operation_fields(Operation::Outline, "first", &catalog).expect("operation fields");

    for identity in [ids::PATH, ids::ADAPTER, ids::PAGE, MATCHING_IDENTITY] {
        assert_has_identity(selected.as_ref(), identity);
    }
    assert_lacks_identity(selected.as_ref(), OTHER_IDENTITY);
}

fn catalog() -> DocumentParameterCatalog {
    let fields = FieldDefSet::builder()
        .field(
            FieldDef::builder(ids::PAGE)
                .process("cli", ProcessStrategy::cli_flag("--page"))
                .validation(FieldValidation::int())
                .default_static(1),
            ExpectedFieldShape::required(),
        )
        .field(
            FieldDef::builder(MATCHING_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--matching"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        )
        .field(
            FieldDef::builder(OTHER_IDENTITY)
                .process("cli", ProcessStrategy::cli_flag("--other"))
                .validation(FieldValidation::int()),
            ExpectedFieldShape::optional(),
        );
    DocumentParameterCatalog::new(
        ["first", "second"],
        fields,
        vec![
            entry(ids::PAGE, None, StandardInputBinding::OutlinePage),
            entry(
                MATCHING_IDENTITY,
                Some("first"),
                StandardInputBinding::OutlineMaxHeadingLevel,
            ),
            entry(
                OTHER_IDENTITY,
                Some("second"),
                StandardInputBinding::OutlineMaxHeadingLevel,
            ),
        ],
    )
    .expect("catalog is valid")
}

fn entry(
    identity: &str,
    adapter_id: Option<&str>,
    binding: StandardInputBinding,
) -> DocumentParameterEntry {
    DocumentParameterEntry::new(
        FieldIdentity::new(identity).expect("valid identity"),
        adapter_id.map(str::to_owned),
        vec![DocumentParameterBinding::StandardInput(binding)],
    )
}

fn assert_has_identity(fields: &FieldDefSet, identity: &str) {
    assert!(
        fields
            .schema_metadata()
            .into_iter()
            .any(|metadata| metadata.identity().as_str() == identity),
        "missing identity {identity}"
    );
}

fn assert_lacks_identity(fields: &FieldDefSet, identity: &str) {
    assert!(
        fields
            .schema_metadata()
            .into_iter()
            .all(|metadata| metadata.identity().as_str() != identity),
        "unexpected identity {identity}"
    );
}
