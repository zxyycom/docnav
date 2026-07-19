use super::*;

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
