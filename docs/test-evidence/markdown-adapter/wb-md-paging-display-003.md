### Case WB-MD-PAGING-DISPLAY-003: Entry paging preserves ref and truncates display

Entry:
- `crates/adapters/markdown/src/paging/tests.rs > entry_paging_preserves_ref_and_truncates_display`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown paging helper 保留 ref 并截断 display”所涉及的稳定行为边界。

Proves:
- 原生入口 `entry_paging_preserves_ref_and_truncates_display` 直接验证“Entry paging preserves ref and truncates display”所描述的结果。
