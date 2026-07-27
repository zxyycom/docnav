### Case WB-CORE-OUTPUT-002: Plain text outcome writes text directly

Entry:
- `crates/docnav/src/output/tests.rs > plain_text_outcome_writes_text_directly`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `plain_text_outcome_writes_text_directly` 直接验证“Plain text outcome writes text directly”所描述的结果。
