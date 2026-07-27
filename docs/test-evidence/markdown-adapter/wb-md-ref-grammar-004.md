### Case WB-MD-REF-GRAMMAR-004: Parse rejects one representative per invalid grammar type

Entry:
- `crates/adapters/markdown/src/markdown/refs/tests.rs > parse_rejects_one_representative_per_invalid_grammar_type`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref grammar 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `parse_rejects_one_representative_per_invalid_grammar_type` 直接验证“Parse rejects one representative per invalid grammar type”所描述的结果。
