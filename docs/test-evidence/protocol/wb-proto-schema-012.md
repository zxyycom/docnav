### Case WB-PROTO-SCHEMA-012: Protocol response public schema rejects undocumented format candidates

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_response_public_schema_rejects_undocumented_format_candidates`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_response_public_schema_rejects_undocumented_format_candidates` 直接验证“Protocol response public schema rejects undocumented format candidates”所描述的结果。
