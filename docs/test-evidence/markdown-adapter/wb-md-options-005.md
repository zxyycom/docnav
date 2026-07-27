### Case WB-MD-OPTIONS-005: Outline rejects out of range max heading level at adapter boundary

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > outline_rejects_out_of_range_max_heading_level_at_adapter_boundary`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown standard input 控制可见粒度”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_rejects_out_of_range_max_heading_level_at_adapter_boundary` 直接验证“Outline rejects out of range max heading level at adapter boundary”所描述的结果。
