use docnav_adapter_contracts::StandardInputBinding;
use docnav_protocol::{Operation, PagedOperation};
use docnav_typed_fields::{
    ExpectedFieldShape, FieldBound, FieldDef, FieldDefSet, FieldDefSetBuildError, FieldIdentity,
    FieldValidation, ProcessStrategy, ValueKind,
};
use serde_json::json;

use super::{
    DocumentParameterBinding, DocumentParameterCatalog, DocumentParameterCatalogBuildError,
    DocumentParameterEntry,
};
use crate::{
    validate_navigation_config_source_value, NavigationConfigSourceLevel,
    NavigationConfigSourceOrigin,
};

const FIRST_IDENTITY: &str = "docnav.parameters.first";
const SECOND_IDENTITY: &str = "docnav.parameters.second";
const KNOWN_ADAPTER: &str = "known-adapter";
const SECOND_ADAPTER: &str = "second-adapter";

#[test]
fn field_set_identity_and_locator_errors_are_preserved() {
    let duplicate_identity = FieldDefSet::builder()
        .field(
            integer_field(FIRST_IDENTITY, "--first"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(FIRST_IDENTITY, "--second"),
            ExpectedFieldShape::optional(),
        );
    let error = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        duplicate_identity,
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect_err("duplicate field identity must fail");
    assert!(matches!(
        error,
        DocumentParameterCatalogBuildError::FieldSet {
            ref source
        } if matches!(source.as_ref(), FieldDefSetBuildError::DuplicateIdentity(_))
    ));

    let duplicate_locator = FieldDefSet::builder()
        .field(
            integer_field(FIRST_IDENTITY, "--same"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(SECOND_IDENTITY, "--same"),
            ExpectedFieldShape::optional(),
        );
    let error = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        duplicate_locator,
        vec![
            entry(FIRST_IDENTITY, None, &[StandardInputBinding::OutlinePage]),
            entry(SECOND_IDENTITY, None, &[StandardInputBinding::FindPage]),
        ],
    )
    .expect_err("duplicate field locator must fail");
    assert!(matches!(
        error,
        DocumentParameterCatalogBuildError::FieldSet {
            ref source
        } if matches!(
            source.as_ref(),
            FieldDefSetBuildError::DuplicateProcessingLocator(_)
        )
    ));
}

#[test]
fn entry_adapter_must_be_known() {
    let error = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            Some("unknown-adapter"),
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect_err("unknown adapter tag must fail");

    assert!(matches!(
        error,
        DocumentParameterCatalogBuildError::UnknownAdapterId {
            identity,
            adapter_id
        } if identity.as_str() == FIRST_IDENTITY && adapter_id == "unknown-adapter"
    ));
}

#[test]
fn catalog_retains_known_adapter_ids_for_config_validation() {
    let catalog = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            Some(KNOWN_ADAPTER),
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect("catalog is valid");

    assert!(catalog.is_known_adapter_id(KNOWN_ADAPTER));
    assert!(!catalog.is_known_adapter_id("unknown-adapter"));
}

#[test]
fn standalone_config_validation_consumes_catalog_scalar_fields() {
    let fields = FieldDefSet::builder().field(
        FieldDef::builder(FIRST_IDENTITY)
            .process(
                "config",
                ProcessStrategy::config_path(["options", KNOWN_ADAPTER, "first"]),
            )
            .validation(
                FieldValidation::int().between(FieldBound::closed(1), FieldBound::closed(6)),
            ),
        ExpectedFieldShape::optional(),
    );
    let catalog = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        fields,
        vec![entry(
            FIRST_IDENTITY,
            Some(KNOWN_ADAPTER),
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect("catalog is valid");
    let path = "/tmp/catalog-project.json";

    let error = validate_navigation_config_source_value(
        NavigationConfigSourceLevel::Project,
        NavigationConfigSourceOrigin::ExplicitCli,
        path,
        json!({
            "options": {
                KNOWN_ADAPTER: {
                    "first": 7
                }
            }
        }),
        &catalog,
    )
    .expect_err("catalog range must reject the config value");
    let details = error.diagnostic().details().to_value();

    assert_eq!(details["field"], format!("options.{KNOWN_ADAPTER}.first"));
    assert_eq!(details["reason"], "range_invalid");
    assert_eq!(details["path"], path);
}

#[test]
fn every_entry_requires_one_unambiguous_binding_per_operation() {
    let missing = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(FIRST_IDENTITY, None, &[])],
    )
    .expect_err("missing binding must fail");
    assert!(matches!(
        missing,
        DocumentParameterCatalogBuildError::MissingBinding { identity }
            if identity.as_str() == FIRST_IDENTITY
    ));

    let duplicate = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[
                StandardInputBinding::OutlinePage,
                StandardInputBinding::OutlinePage,
            ],
        )],
    )
    .expect_err("duplicate binding must fail");
    assert!(matches!(
        duplicate,
        DocumentParameterCatalogBuildError::DuplicateBinding {
            identity,
            binding: DocumentParameterBinding::StandardInput(
                StandardInputBinding::OutlinePage
            )
        } if identity.as_str() == FIRST_IDENTITY
    ));

    let conflicting_target = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[
                StandardInputBinding::OutlinePage,
                StandardInputBinding::OutlineLimit,
            ],
        )],
    )
    .expect_err("two targets for one operation must fail");
    assert!(matches!(
        conflicting_target,
        DocumentParameterCatalogBuildError::InvalidOperationTarget {
            identity,
            operation: Operation::Outline
        } if identity.as_str() == FIRST_IDENTITY
    ));
}

#[test]
fn standard_input_targets_are_unique_for_overlapping_adapter_scopes() {
    let fields = FieldDefSet::builder()
        .field(
            integer_field(FIRST_IDENTITY, "--first"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(SECOND_IDENTITY, "--second"),
            ExpectedFieldShape::optional(),
        );
    let error = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER, SECOND_ADAPTER],
        fields,
        vec![
            entry(FIRST_IDENTITY, None, &[StandardInputBinding::OutlinePage]),
            entry(
                SECOND_IDENTITY,
                Some(KNOWN_ADAPTER),
                &[StandardInputBinding::OutlinePage],
            ),
        ],
    )
    .expect_err("common and adapter-scoped fields cannot target the same input");
    assert!(matches!(
        error,
        DocumentParameterCatalogBuildError::DuplicateOperationTarget {
            previous_identity,
            current_identity,
            binding: DocumentParameterBinding::StandardInput(
                StandardInputBinding::OutlinePage
            ),
        } if previous_identity.as_str() == FIRST_IDENTITY
            && current_identity.as_str() == SECOND_IDENTITY
    ));

    let fields = FieldDefSet::builder()
        .field(
            integer_field(FIRST_IDENTITY, "--first"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(SECOND_IDENTITY, "--second"),
            ExpectedFieldShape::optional(),
        );
    DocumentParameterCatalog::new(
        [KNOWN_ADAPTER, SECOND_ADAPTER],
        fields,
        vec![
            entry(
                FIRST_IDENTITY,
                Some(KNOWN_ADAPTER),
                &[StandardInputBinding::OutlinePage],
            ),
            entry(
                SECOND_IDENTITY,
                Some(SECOND_ADAPTER),
                &[StandardInputBinding::OutlinePage],
            ),
        ],
    )
    .expect("different exact adapter scopes are mutually exclusive");
}

#[test]
fn binding_value_kind_must_match_the_field_definition() {
    let field = FieldDef::builder(FIRST_IDENTITY)
        .process("cli", ProcessStrategy::cli_flag("--first"))
        .validation(FieldValidation::string());
    let error = catalog(
        field,
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[StandardInputBinding::OutlineMaxHeadingLevel],
        )],
    )
    .expect_err("binding kind mismatch must fail");

    assert!(matches!(
        error,
        DocumentParameterCatalogBuildError::BindingValueKindMismatch {
            identity,
            binding: DocumentParameterBinding::StandardInput(
                StandardInputBinding::OutlineMaxHeadingLevel
            ),
            expected: ValueKind::Integer,
            actual: ValueKind::String,
        } if identity.as_str() == FIRST_IDENTITY
    ));
}

#[test]
fn field_and_entry_associations_are_total_and_unique() {
    let unknown_entry = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            SECOND_IDENTITY,
            None,
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect_err("entry for an unknown field must fail");
    assert!(matches!(
        unknown_entry,
        DocumentParameterCatalogBuildError::UnknownFieldAssociation { identity }
            if identity.as_str() == SECOND_IDENTITY
    ));

    let duplicate_entry = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![
            entry(FIRST_IDENTITY, None, &[StandardInputBinding::OutlinePage]),
            entry(FIRST_IDENTITY, None, &[StandardInputBinding::FindPage]),
        ],
    )
    .expect_err("duplicate entry association must fail");
    assert!(matches!(
        duplicate_entry,
        DocumentParameterCatalogBuildError::DuplicateEntryAssociation { identity }
            if identity.as_str() == FIRST_IDENTITY
    ));

    let fields = FieldDefSet::builder()
        .field(
            integer_field(FIRST_IDENTITY, "--first"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(SECOND_IDENTITY, "--second"),
            ExpectedFieldShape::optional(),
        );
    let missing_entry = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        fields,
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect_err("field without an entry must fail");
    assert!(matches!(
        missing_entry,
        DocumentParameterCatalogBuildError::MissingEntryAssociation { identity }
            if identity.as_str() == SECOND_IDENTITY
    ));
}

#[test]
fn operation_applicability_is_derived_from_closed_bindings() {
    let catalog = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            Some(KNOWN_ADAPTER),
            &[
                StandardInputBinding::OutlineMaxHeadingLevel,
                StandardInputBinding::FindMaxHeadingLevel,
            ],
        )],
    )
    .expect("catalog is valid");
    let entry = catalog.entries().first().expect("catalog entry");

    assert_eq!(entry.adapter_id(), Some(KNOWN_ADAPTER));
    assert_eq!(
        entry.operations().collect::<Vec<_>>(),
        vec![Operation::Outline, Operation::Find]
    );
    assert!(entry.applies_to(Operation::Outline));
    assert!(entry.applies_to(Operation::Find));
    assert!(!entry.applies_to(Operation::Read));
}

#[test]
fn navigation_and_core_only_bindings_enforce_their_target_value_kinds() {
    let boolean = FieldDef::builder(FIRST_IDENTITY)
        .process("cli", ProcessStrategy::cli_flag("--first"))
        .validation(FieldValidation::boolean());
    DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        FieldDefSet::builder().field(boolean, ExpectedFieldShape::optional()),
        vec![DocumentParameterEntry::new(
            FieldIdentity::new(FIRST_IDENTITY).expect("test identity is valid"),
            None,
            vec![DocumentParameterBinding::PaginationEnabled(
                PagedOperation::Outline,
            )],
        )],
    )
    .expect("pagination binding accepts Boolean");

    let string = FieldDef::builder(FIRST_IDENTITY)
        .process("cli", ProcessStrategy::cli_flag("--first"))
        .validation(FieldValidation::string());
    DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        FieldDefSet::builder().field(string, ExpectedFieldShape::optional()),
        vec![DocumentParameterEntry::new(
            FieldIdentity::new(FIRST_IDENTITY).expect("test identity is valid"),
            None,
            vec![DocumentParameterBinding::OutputMode(Operation::Info)],
        )],
    )
    .expect("output binding accepts String");
}

#[test]
fn operation_projection_borrows_the_canonical_field_facts() {
    let catalog = catalog(
        integer_field(FIRST_IDENTITY, "--first"),
        vec![entry(
            FIRST_IDENTITY,
            None,
            &[StandardInputBinding::OutlinePage],
        )],
    )
    .expect("catalog is valid");
    let canonical = catalog
        .fields()
        .field(&FieldIdentity::new(FIRST_IDENTITY).expect("valid identity"))
        .expect("canonical field");
    let projected = catalog
        .operation_fields(Operation::Outline)
        .next()
        .expect("projected field");

    assert!(std::ptr::eq(canonical, projected));
    assert!(catalog.operation_fields(Operation::Read).next().is_none());
}

#[test]
fn selected_operation_projection_includes_common_and_exact_adapter_fields_only() {
    let common_identity = "docnav.parameters.common";
    let matching_identity = "docnav.parameters.matching";
    let other_identity = "docnav.parameters.other";
    let wrong_operation_identity = "docnav.parameters.wrong_operation";
    let fields = FieldDefSet::builder()
        .field(
            integer_field(common_identity, "--common"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(matching_identity, "--matching"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(other_identity, "--other"),
            ExpectedFieldShape::optional(),
        )
        .field(
            integer_field(wrong_operation_identity, "--wrong-operation"),
            ExpectedFieldShape::optional(),
        );
    let catalog = DocumentParameterCatalog::new(
        [KNOWN_ADAPTER, SECOND_ADAPTER],
        fields,
        vec![
            entry(common_identity, None, &[StandardInputBinding::OutlinePage]),
            entry(
                matching_identity,
                Some(KNOWN_ADAPTER),
                &[StandardInputBinding::OutlineLimit],
            ),
            entry(
                other_identity,
                Some(SECOND_ADAPTER),
                &[StandardInputBinding::OutlineLimit],
            ),
            entry(
                wrong_operation_identity,
                Some(KNOWN_ADAPTER),
                &[StandardInputBinding::ReadPage],
            ),
        ],
    )
    .expect("catalog is valid");

    let identities = catalog
        .selected_operation_fields(KNOWN_ADAPTER, Operation::Outline)
        .map(|field| field.identity().as_str())
        .collect::<Vec<_>>();

    assert_eq!(identities, vec![common_identity, matching_identity]);
}

fn catalog<T: 'static>(
    field: docnav_typed_fields::FieldDefBuilder<T>,
    entries: Vec<DocumentParameterEntry>,
) -> Result<DocumentParameterCatalog, DocumentParameterCatalogBuildError> {
    DocumentParameterCatalog::new(
        [KNOWN_ADAPTER],
        FieldDefSet::builder().field(field, ExpectedFieldShape::optional()),
        entries,
    )
}

fn integer_field(identity: &str, flag: &str) -> docnav_typed_fields::FieldDefBuilder<i64> {
    FieldDef::builder(identity)
        .process("cli", ProcessStrategy::cli_flag(flag))
        .validation(FieldValidation::int())
}

fn entry(
    identity: &str,
    adapter_id: Option<&str>,
    bindings: &[StandardInputBinding],
) -> DocumentParameterEntry {
    DocumentParameterEntry::new(
        FieldIdentity::new(identity).expect("test identity is valid"),
        adapter_id.map(str::to_owned),
        bindings
            .iter()
            .copied()
            .map(DocumentParameterBinding::StandardInput)
            .collect(),
    )
}
