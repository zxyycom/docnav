### Case WB-TYPED-FIELDS-005: Required and enum constraints are driven by field declarations

Entry:
- `crates/shared/typed-fields/src/tests/field_model.rs > required_and_enum_constraints_are_driven_by_field_declarations`

Contract:
- `docs/architecture.md` 定义或约束“Typed field definition core 保持字段级不变量”所涉及的稳定行为边界。

Proves:
- 原生入口 `required_and_enum_constraints_are_driven_by_field_declarations` 直接验证“Required and enum constraints are driven by field declarations”所描述的结果。
