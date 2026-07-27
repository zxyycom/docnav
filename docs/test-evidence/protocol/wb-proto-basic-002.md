### Case WB-PROTO-BASIC-002: Positive integer constructors reject zero

Entry:
- `crates/shared/protocol/src/tests/basic.rs > positive_integer_constructors_reject_zero`

Contract:
- `docs/protocol.md` 定义或约束“Protocol 基础类型和 envelope 规则稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `positive_integer_constructors_reject_zero` 直接验证“Positive integer constructors reject zero”所描述的结果。
