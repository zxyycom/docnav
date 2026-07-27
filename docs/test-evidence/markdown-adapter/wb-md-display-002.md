### Case WB-MD-DISPLAY-002: Outline entries include heading title

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > outline_entries_include_heading_title`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline/find display 保留可读文本”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_entries_include_heading_title` 直接验证“Outline entries include heading title”所描述的结果。
