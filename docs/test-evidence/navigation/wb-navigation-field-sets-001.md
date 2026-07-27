### Case WB-NAVIGATION-FIELD-SETS-001: Selected field set follows closed catalog applicability

Entry:
- `crates/shared/navigation/src/parameters/fields/tests.rs > selected_fields_combine_fixed_inputs_with_catalog_projection`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Selected field set follows closed catalog applicability”所涉及的稳定行为边界。

Proves:
- The selected operation field set combines fixed operation inputs with the core-authored parameter catalog projection.
- Adapter-scoped catalog fields are included only for the selected adapter；fields scoped to another adapter are excluded.
