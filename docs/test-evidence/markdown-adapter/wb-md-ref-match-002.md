### Case WB-MD-REF-MATCH-002: Matches exact line level

Entry:
- `crates/adapters/markdown/src/markdown/refs/tests.rs > matches_exact_line_level`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown parsed ref 精确匹配 heading 坐标”所涉及的稳定行为边界。

Proves:
- 原生入口 `matches_exact_line_level` 直接验证“Matches exact line level”所描述的结果。
