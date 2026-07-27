### Case WB-PROTO-SCHEMA-010: Protocol auto read contract rejects status error and extra fields

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_auto_read_contract_rejects_status_error_and_extra_fields`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_auto_read_contract_rejects_status_error_and_extra_fields` 直接验证“Protocol auto read contract rejects status error and extra fields”所描述的结果。
