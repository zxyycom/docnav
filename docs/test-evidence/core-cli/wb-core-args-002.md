### Case WB-CORE-ARGS-002: Auto read rejects missing duplicate and inapplicable input structurally

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > auto_read_rejects_missing_duplicate_and_inapplicable_input_structurally`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `auto_read_rejects_missing_duplicate_and_inapplicable_input_structurally` 直接验证“Auto read rejects missing duplicate and inapplicable input structurally”所描述的结果。
