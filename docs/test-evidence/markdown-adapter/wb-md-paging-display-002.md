### Case WB-MD-PAGING-DISPLAY-002: Read paging counts unicode characters

Entry:
- `crates/adapters/markdown/src/paging/tests.rs > read_paging_counts_unicode_characters`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown paging helper 保留 ref 并截断 display”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_paging_counts_unicode_characters` 直接验证“Read paging counts unicode characters”所描述的结果。
