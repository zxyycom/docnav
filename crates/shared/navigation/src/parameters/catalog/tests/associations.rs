use super::*;

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
