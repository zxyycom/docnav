### Case WB-MD-DOCHEAD-009: Read document head returns original markdown and paginates unicode

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > read_document_head_returns_original_markdown_and_paginates_unicode`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head read/find roundtrip 和分页稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_document_head_returns_original_markdown_and_paginates_unicode` 直接验证“Read document head returns original markdown and paginates unicode”所描述的结果。
