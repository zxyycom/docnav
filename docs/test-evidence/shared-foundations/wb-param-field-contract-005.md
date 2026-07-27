### Case WB-PARAM-FIELD-CONTRACT-005: Set build rejects duplicate and invalid source locators

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > set_build_rejects_duplicate_and_invalid_source_locators`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `set_build_rejects_duplicate_and_invalid_source_locators` 直接验证“Set build rejects duplicate and invalid source locators”所描述的结果。
