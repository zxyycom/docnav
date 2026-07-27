### Case WB-CORE-ARGS-006: Unused known argument value is rejected before execution

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > unused_known_argument_value_is_rejected_before_execution`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `unused_known_argument_value_is_rejected_before_execution` 直接验证“Unused known argument value is rejected before execution”所描述的结果。
