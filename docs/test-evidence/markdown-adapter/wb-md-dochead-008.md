### Case WB-MD-DOCHEAD-008: Find falls back to full document when no heading is visible

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > find_falls_back_to_full_document_when_no_heading_is_visible`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head read/find roundtrip 和分页稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `find_falls_back_to_full_document_when_no_heading_is_visible` 直接验证“Find falls back to full document when no heading is visible”所描述的结果。
