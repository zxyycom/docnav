### Case WB-PROTO-BASIC-007: Base result constructors omit auto read

Entry:
- `crates/shared/protocol/src/tests/basic.rs > base_result_constructors_omit_auto_read`

Contract:
- `docs/protocol.md` 定义或约束“Protocol 基础类型和 envelope 规则稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `base_result_constructors_omit_auto_read` 直接验证“Base result constructors omit auto read”所描述的结果。
