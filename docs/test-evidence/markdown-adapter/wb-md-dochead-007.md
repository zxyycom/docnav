### Case WB-MD-DOCHEAD-007: Find match before first visible heading uses document head ref

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > find_match_before_first_visible_heading_uses_document_head_ref`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head read/find roundtrip 和分页稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `find_match_before_first_visible_heading_uses_document_head_ref` 直接验证“Find match before first visible heading uses document head ref”所描述的结果。
