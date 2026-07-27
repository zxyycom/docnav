# Claim CLAIM-WB-MD-ERROR-001: Markdown adapter document error 稳定

Topic: `markdown-adapter`
Owner ref: `docs/adapters/markdown.md#错误分类`

Statement:
- Markdown input that is not valid UTF-8 returns the adapter-owned encoding failure.

Observations:
- non-UTF-8 document 返回稳定 encoding error。

Supported by:
- `cargo|docnav-markdown:test:adapter|options_error_display::non_utf8_document_returns_stable_encoding_error`
