### Case WB-TEXT-COST-002: Line cost counts empty unicode and trailing newline

Entry:
- `crates/shared/text-cost/src/tests.rs > line_cost_counts_empty_unicode_and_trailing_newline`

Contract:
- `docs/architecture.md` 定义或约束“Shared text cost helper 保持纯文本边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `line_cost_counts_empty_unicode_and_trailing_newline` 直接验证“Line cost counts empty unicode and trailing newline”所描述的结果。
