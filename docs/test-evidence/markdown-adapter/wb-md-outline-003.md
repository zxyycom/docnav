### Case WB-MD-OUTLINE-003: Outline refs consistent under different max heading level

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > outline_refs_consistent_under_different_max_heading_level`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline ref 和 display 语义稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_refs_consistent_under_different_max_heading_level` 直接验证“Outline refs consistent under different max heading level”所描述的结果。
