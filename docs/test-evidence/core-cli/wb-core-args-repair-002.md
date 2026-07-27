### Case WB-CORE-ARGS-REPAIR-002: Unknown document argument protocol error has repair context

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/protocol_errors.rs > unknown_document_argument_protocol_error_has_repair_context`

Contract:
- `docs/cli.md` 定义或约束“Core parser input diagnostics expose protocol repair context”所涉及的稳定行为边界。

Proves:
- 原生入口 `unknown_document_argument_protocol_error_has_repair_context` 直接验证“Unknown document argument protocol error has repair context”所描述的结果。
