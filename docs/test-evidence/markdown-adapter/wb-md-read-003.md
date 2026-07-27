### Case WB-MD-READ-003: Doc full still resolves to full document

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > doc_full_still_resolves_to_full_document`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown read resolve 和 doc:full ref 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `doc_full_still_resolves_to_full_document` 直接验证“Doc full still resolves to full document”所描述的结果。
