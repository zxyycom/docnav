### Case WB-MD-DOCHEAD-005: Outline omits document head for empty or whitespace only prefix

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_omits_document_head_for_empty_or_whitespace_only_prefix`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown document head outline eligibility 和 raw facts 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_omits_document_head_for_empty_or_whitespace_only_prefix` 直接验证“Outline omits document head for empty or whitespace only prefix”所描述的结果。
