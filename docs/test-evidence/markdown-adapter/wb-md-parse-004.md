### Case WB-MD-PARSE-004: Read section ends at next same or higher heading

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > read_section_ends_at_next_same_or_higher_heading`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown parser 忽略非 heading 结构”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_section_ends_at_next_same_or_higher_heading` 直接验证“Read section ends at next same or higher heading”所描述的结果。
