### Case BB-CORE-FAIL-001: Candidate probe failure 投影为格式候选摘要

Entry:
- `test/smoke/core/cases/failures.ts > smoke task CORE-FAIL-001`

Contract:
- `docs/adapter-contract.md` 定义或约束“Candidate probe failure 投影为格式候选摘要”所涉及的稳定行为边界。

Proves:
- candidate discovery 阶段的 built-in adapter probe failure 被报告为 `FORMAT_UNKNOWN` candidate summary。
- candidate failure 不会被折叠成 selected adapter layer failure。
- 未显式声明 adapter 的 automatic discovery 全部 probe 失败时，candidate failures 从属于 primary diagnostic details。
