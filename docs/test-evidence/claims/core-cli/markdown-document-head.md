# Claim CLAIM-CLI-MARKDOWN-DOCUMENT-HEAD-001: Markdown document head 通过真实 CLI 输出模式可观察

Topic: `core-cli`
Owner ref: `docs/adapters/markdown.md#document-head`

Statement:
- Structured Markdown outline exposes eligible document-head content as HEAD:leading before visible headings.

Observations:
- 真实 CLI fixture 包含 YAML frontmatter、普通前导正文和可见 heading 时，structured outline 在 heading entries 前暴露 `HEAD:leading`。
- `protocol-json` 验证 raw document head entry facts：非空 `label`、`kind = document_head`、`location.line_start`、`metadata.document_region = leading` 和缺少 readable-only `display`。
- `readable-view` 验证 display、成本摘要和 read content block 由内置 renderer 从同一 `ProtocolResponse` 的 raw facts 与 read result 派生。
- 通过 `HEAD:leading` 执行 read 返回 document head 原文，`content_type` 为 `text/markdown`，并保留 frontmatter delimiter 与普通前导正文。

Supported by:
- `smoke|core:real-markdown-link-chain|CORE-MD-DOCHEAD-001`
