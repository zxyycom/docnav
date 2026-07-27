### Case BB-CORE-REF-001: Adapter ref 错误穿过 Core

Entry:
- `test/smoke/core/cases/real-markdown.ts > smoke task CORE-REF-001`

Contract:
- `docs/ref-contract.md` 定义或约束“Adapter ref 错误穿过 Core”所涉及的稳定行为边界。

Proves:
- 被选中 adapter 拒绝的 ref 会从 core 返回稳定 protocol failure。
- `protocol-json` 承载错误时，stderr 不输出 JSON payload。
