### Case WB-MD-REF-004: Read reports ref not found for canonical no match

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > read_reports_ref_not_found_for_canonical_no_match`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref 错误区分 invalid 和 not-found”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_reports_ref_not_found_for_canonical_no_match` 直接验证“Read reports ref not found for canonical no match”所描述的结果。
