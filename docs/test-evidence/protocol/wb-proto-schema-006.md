### Case WB-PROTO-SCHEMA-006: Probe schema rejects missing reasons and bad confidence

Entry:
- `crates/shared/protocol/src/tests/schema.rs > probe_schema_rejects_missing_reasons_and_bad_confidence`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `probe_schema_rejects_missing_reasons_and_bad_confidence` 直接验证“Probe schema rejects missing reasons and bad confidence”所描述的结果。
