### Case WB-PARAM-FIELD-CONTRACT-007: Field lookup uses canonical final value validation

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > field_lookup_uses_canonical_final_value_validation`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `field_lookup_uses_canonical_final_value_validation` 直接验证“Field lookup uses canonical final value validation”所描述的结果。
