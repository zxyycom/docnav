### Case WB-MD-DISPLAY-003: Find entry contains match snippet

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > find_entry_contains_match_snippet`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline/find display 保留可读文本”所涉及的稳定行为边界。

Proves:
- 原生入口 `find_entry_contains_match_snippet` 直接验证“Find entry contains match snippet”所描述的结果。
