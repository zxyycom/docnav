### Case WB-MD-REF-005: Structure snapshot ref is evaluated against current document

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > structure_snapshot_ref_is_evaluated_against_current_document`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref 错误区分 invalid 和 not-found”所涉及的稳定行为边界。

Proves:
- 原生入口 `structure_snapshot_ref_is_evaluated_against_current_document` 直接验证“Structure snapshot ref is evaluated against current document”所描述的结果。
