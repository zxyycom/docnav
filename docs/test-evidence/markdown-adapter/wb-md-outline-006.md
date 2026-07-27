### Case WB-MD-OUTLINE-006: Deep heading can be filtered to full document

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > deep_heading_can_be_filtered_to_full_document`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline ref 和 display 语义稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `deep_heading_can_be_filtered_to_full_document` 直接验证“Deep heading can be filtered to full document”所描述的结果。
