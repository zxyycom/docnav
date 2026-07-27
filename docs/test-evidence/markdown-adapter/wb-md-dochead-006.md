### Case WB-MD-DOCHEAD-006: Outline keeps frontmatter pseudo heading fence pseudo heading and hr in document head

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_keeps_frontmatter_pseudo_heading_fence_pseudo_heading_and_hr_in_document_head`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head outline eligibility 和 raw facts 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_keeps_frontmatter_pseudo_heading_fence_pseudo_heading_and_hr_in_document_head` 直接验证“Outline keeps frontmatter pseudo heading fence pseudo heading and hr in document head”所描述的结果。
