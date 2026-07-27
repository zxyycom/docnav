### Case WB-PARAM-FIELD-CONTRACT-006: Merge strategy is canonical field metadata

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > merge_strategy_is_canonical_field_metadata`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `merge_strategy_is_canonical_field_metadata` 直接验证“Merge strategy is canonical field metadata”所描述的结果。
