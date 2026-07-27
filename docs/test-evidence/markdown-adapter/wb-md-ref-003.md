### Case WB-MD-REF-003: Read reports ref invalid for grammar outside refs

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > read_reports_ref_invalid_for_grammar_outside_refs`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref 错误区分 invalid 和 not-found”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_reports_ref_invalid_for_grammar_outside_refs` 直接验证“Read reports ref invalid for grammar outside refs”所描述的结果。
