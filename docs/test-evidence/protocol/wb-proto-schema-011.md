### Case WB-PROTO-SCHEMA-011: Protocol auto read contract rejects unstructured read and info placement

Entry:
- `crates/shared/protocol/src/tests/schema.rs > protocol_auto_read_contract_rejects_unstructured_read_and_info_placement`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_auto_read_contract_rejects_unstructured_read_and_info_placement` 直接验证“Protocol auto read contract rejects unstructured read and info placement”所描述的结果。
