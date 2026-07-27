### Case WB-MD-OPTIONS-004: Outline does not default a missing max heading level

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > outline_does_not_default_a_missing_max_heading_level`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown standard input 控制可见粒度”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_does_not_default_a_missing_max_heading_level` 直接验证“Outline does not default a missing max heading level”所描述的结果。
