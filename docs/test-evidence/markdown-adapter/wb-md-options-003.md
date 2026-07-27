### Case WB-MD-OPTIONS-003: Outline consumes max heading level from standard input

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > outline_consumes_max_heading_level_from_standard_input`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown standard input 控制可见粒度”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_consumes_max_heading_level_from_standard_input` 直接验证“Outline consumes max heading level from standard input”所描述的结果。
