### Case WB-CORE-ARGS-REPAIR-004: Unsupported info page protocol error has repair context

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/protocol_errors.rs > unsupported_info_page_protocol_error_has_repair_context`

Contract:
- `docs/cli.md` 定义或约束“Core parser input diagnostics expose protocol repair context”所涉及的稳定行为边界。

Proves:
- 原生入口 `unsupported_info_page_protocol_error_has_repair_context` 直接验证“Unsupported info page protocol error has repair context”所描述的结果。
