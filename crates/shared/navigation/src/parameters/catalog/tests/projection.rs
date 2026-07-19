use super::*;

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
