### Case WB-TYPED-FIELDS-002: Builder exposes schema metadata and validates values

Entry:
- `crates/shared/typed-fields/src/tests/field_model.rs > builder_exposes_schema_metadata_and_validates_values`

Contract:
- `docs/architecture.md` 定义或约束“Typed field definition core 保持字段级不变量”所涉及的稳定行为边界。

Proves:
- 原生入口 `builder_exposes_schema_metadata_and_validates_values` 直接验证“Builder exposes schema metadata and validates values”所描述的结果。
