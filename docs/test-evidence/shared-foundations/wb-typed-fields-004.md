### Case WB-TYPED-FIELDS-004: Validation failures keep field attribution

Entry:
- `crates/shared/typed-fields/src/tests/field_model.rs > validation_failures_keep_field_attribution`

Contract:
- `docs/architecture.md` 定义或约束“Typed field definition core 保持字段级不变量”所涉及的稳定行为边界。

Proves:
- 原生入口 `validation_failures_keep_field_attribution` 直接验证“Validation failures keep field attribution”所描述的结果。
