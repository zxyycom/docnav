# Claim CLAIM-NAVIGATION-SELECTED-FIELD-SET-001: Selected field set follows closed catalog applicability

Topic: `navigation`
Owner ref: `docs/navigation-input-resolution.md#selected-operation-catalog-view`

Statement:
- The selected operation field set combines fixed operation inputs with only the applicable core-authored catalog entries.

Observations:
- The selected operation field set combines fixed operation inputs with the core-authored parameter catalog projection.
- Adapter-scoped catalog fields are included only for the selected adapter；fields scoped to another adapter are excluded.

Supported by:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::fields::tests::selected_fields_combine_fixed_inputs_with_catalog_projection`
