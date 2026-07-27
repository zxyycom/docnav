### Case WB-PARAM-FIELD-CONTRACT-002: Canonical processing metadata exposes source locators

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > canonical_processing_metadata_exposes_source_locators`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `canonical_processing_metadata_exposes_source_locators` 直接验证“Canonical processing metadata exposes source locators”所描述的结果。
