### Case WB-CORE-ARGS-008: Extra document positional is rejected

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > extra_document_positional_is_rejected`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `extra_document_positional_is_rejected` 直接验证“Extra document positional is rejected”所描述的结果。
