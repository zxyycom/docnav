### Case WB-PROTO-SCHEMA-002: Parses protocol fixtures into shared types

Entry:
- `crates/shared/protocol/src/tests/schema.rs > parses_protocol_fixtures_into_shared_types`

Contract:
- `docs/protocol.md` 定义或约束“Protocol fixtures 和 schema constraints 被实现测试消费”所涉及的稳定行为边界。

Proves:
- 原生入口 `parses_protocol_fixtures_into_shared_types` 直接验证“Parses protocol fixtures into shared types”所描述的结果。
