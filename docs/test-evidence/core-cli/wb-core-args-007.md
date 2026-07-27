### Case WB-CORE-ARGS-007: Unknown document argument is rejected

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > unknown_document_argument_is_rejected`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `unknown_document_argument_is_rejected` 直接验证“Unknown document argument is rejected”所描述的结果。
