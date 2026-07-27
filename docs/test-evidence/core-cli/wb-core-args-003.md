### Case WB-CORE-ARGS-003: Max heading level is rejected for unsupported operations

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > max_heading_level_is_rejected_for_unsupported_operations`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `max_heading_level_is_rejected_for_unsupported_operations` 直接验证“Max heading level is rejected for unsupported operations”所描述的结果。
