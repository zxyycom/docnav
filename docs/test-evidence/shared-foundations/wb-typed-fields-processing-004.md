### Case WB-TYPED-FIELDS-PROCESSING-004: Processing id try from rejects empty value

Entry:
- `crates/shared/typed-fields/src/tests/processing.rs > processing_id_try_from_rejects_empty_value`

Contract:
- `docs/architecture.md` 定义或约束“Typed field processing build 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `processing_id_try_from_rejects_empty_value` 直接验证“Processing id try from rejects empty value”所描述的结果。
