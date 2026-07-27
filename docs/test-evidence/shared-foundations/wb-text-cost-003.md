### Case WB-TEXT-COST-003: Byte cost counts utf8 bytes

Entry:
- `crates/shared/text-cost/src/tests.rs > byte_cost_counts_utf8_bytes`

Contract:
- `docs/architecture.md` 定义或约束“Shared text cost helper 保持纯文本边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `byte_cost_counts_utf8_bytes` 直接验证“Byte cost counts utf8 bytes”所描述的结果。
