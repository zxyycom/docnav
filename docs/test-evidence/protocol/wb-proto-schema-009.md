### Case WB-PROTO-SCHEMA-009: Protocol auto read contract accepts exact outline and find success objects

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_auto_read_contract_accepts_exact_outline_and_find_success_objects`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_auto_read_contract_accepts_exact_outline_and_find_success_objects` 直接验证“Protocol auto read contract accepts exact outline and find success objects”所描述的结果。
