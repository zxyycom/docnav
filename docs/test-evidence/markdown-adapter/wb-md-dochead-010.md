### Case WB-MD-DOCHEAD-010: Read document head preserves yaml delimiters and leading text

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > read_document_head_preserves_yaml_delimiters_and_leading_text`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head read/find roundtrip 和分页稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_document_head_preserves_yaml_delimiters_and_leading_text` 直接验证“Read document head preserves yaml delimiters and leading text”所描述的结果。
