### Case WB-TYPED-FIELDS-PROCESSING-003: Processing build rejects empty processing id

Entry:
- `crates/shared/typed-fields/src/tests/processing.rs > processing_build_rejects_empty_processing_id`

Contract:
- `docs/architecture.md` 定义或约束“Typed field processing build 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `processing_build_rejects_empty_processing_id` 直接验证“Processing build rejects empty processing id”所描述的结果。
