### Case BB-CORE-MD-DOCHEAD-001: Markdown document head 通过真实 CLI 输出模式可观察

Entry:
- `test/smoke/core/cases/real-markdown.ts > smoke task CORE-MD-DOCHEAD-001`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head 通过真实 CLI 输出模式可观察”所涉及的稳定行为边界。

Proves:
- 真实 CLI fixture 包含 YAML frontmatter、普通前导正文和可见 heading 时，structured outline 在 heading entries 前暴露 `HEAD:leading`。
- `protocol-json` 验证 raw document head entry facts：非空 `label`、`kind = document_head`、`location.line_start`、`metadata.document_region = leading` 和缺少 readable-only `display`。
- `readable-view` 验证 display、成本摘要和 read content block 由内置 renderer 从同一 `ProtocolResponse` 的 raw facts 与 read result 派生。
- 通过 `HEAD:leading` 执行 read 返回 document head 原文，`content_type` 为 `text/markdown`，并保留 frontmatter delimiter 与普通前导正文。
