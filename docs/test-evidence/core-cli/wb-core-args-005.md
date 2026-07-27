### Case WB-CORE-ARGS-005: Duplicate generated single value flag is rejected structurally

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > duplicate_generated_single_value_flag_is_rejected_structurally`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `duplicate_generated_single_value_flag_is_rejected_structurally` 直接验证“Duplicate generated single value flag is rejected structurally”所描述的结果。
