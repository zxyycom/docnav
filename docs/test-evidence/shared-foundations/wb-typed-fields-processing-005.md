### Case WB-TYPED-FIELDS-PROCESSING-005: Field build rejects duplicate processing id

Entry:
- `crates/shared/typed-fields/src/tests/processing.rs > field_build_rejects_duplicate_processing_id`

Contract:
- `docs/architecture.md` 定义或约束“Typed field processing build 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `field_build_rejects_duplicate_processing_id` 直接验证“Field build rejects duplicate processing id”所描述的结果。
