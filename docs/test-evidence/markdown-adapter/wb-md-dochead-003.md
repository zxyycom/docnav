### Case WB-MD-DOCHEAD-003: Outline exposes document head before visible headings when nonblank

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_exposes_document_head_before_visible_headings_when_nonblank`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head outline eligibility 和 raw facts 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_exposes_document_head_before_visible_headings_when_nonblank` 直接验证“Outline exposes document head before visible headings when nonblank”所描述的结果。
