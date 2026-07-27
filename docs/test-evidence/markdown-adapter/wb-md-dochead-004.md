### Case WB-MD-DOCHEAD-004: Outline exposes document head for frontmatter only or plain lead

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_exposes_document_head_for_frontmatter_only_or_plain_lead`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head outline eligibility 和 raw facts 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_exposes_document_head_for_frontmatter_only_or_plain_lead` 直接验证“Outline exposes document head for frontmatter only or plain lead”所描述的结果。
