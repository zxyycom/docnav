### Case WB-PROTO-BASIC-009: Failure response rules preserve or null operation

Entry:
- `crates/shared/protocol/src/tests/basic.rs > failure_response_rules_preserve_or_null_operation`

Contract:
- `docs/protocol.md` 定义或约束“Protocol 基础类型和 envelope 规则稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `failure_response_rules_preserve_or_null_operation` 直接验证“Failure response rules preserve or null operation”所描述的结果。
