### Case WB-MD-ERROR-001: Markdown adapter document error 稳定

Entry:
- `crates/adapters/markdown/tests/adapter/options_error_display.rs > non_utf8_document_returns_stable_encoding_error`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown adapter document error 稳定”所涉及的稳定行为边界。

Proves:
- non-UTF-8 document 返回稳定 encoding error。
