### Case WB-PROTO-SCHEMA-004: Protocol request contract rejects schema backed field failures

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_request_contract_rejects_schema_backed_field_failures`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_request_contract_rejects_schema_backed_field_failures` 直接验证“Protocol request contract rejects schema backed field failures”所描述的结果。
