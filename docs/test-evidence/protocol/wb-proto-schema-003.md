### Case WB-PROTO-SCHEMA-003: Protocol request schema rejects an empty required string

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_request_schema_rejects_an_empty_required_string`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_request_schema_rejects_an_empty_required_string` 直接验证“Protocol request schema rejects an empty required string”所描述的结果。
