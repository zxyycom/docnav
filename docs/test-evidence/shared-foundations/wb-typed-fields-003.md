### Case WB-TYPED-FIELDS-003: Json validation accepts any json value including null

Entry:
- `crates/shared/typed-fields/src/tests/field_model.rs > json_validation_accepts_any_json_value_including_null`

Contract:
- `docs/architecture.md` 定义或约束“Typed field definition core 保持字段级不变量”所涉及的稳定行为边界。

Proves:
- 原生入口 `json_validation_accepts_any_json_value_including_null` 直接验证“Json validation accepts any json value including null”所描述的结果。
