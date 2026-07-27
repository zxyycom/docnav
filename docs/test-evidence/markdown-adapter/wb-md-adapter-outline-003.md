### Case WB-MD-ADAPTER-OUTLINE-003: Outline falls back to full document for no visible heading

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_falls_back_to_full_document_for_no_visible_heading`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown adapter outline 默认层级和 fallback 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_falls_back_to_full_document_for_no_visible_heading` 直接验证“Outline falls back to full document for no visible heading”所描述的结果。
