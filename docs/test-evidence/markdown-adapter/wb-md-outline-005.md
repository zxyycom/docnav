### Case WB-MD-OUTLINE-005: Outline entry handles whitespace only title

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > outline_entry_handles_whitespace_only_title`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline ref 和 display 语义稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_entry_handles_whitespace_only_title` 直接验证“Outline entry handles whitespace only title”所描述的结果。
