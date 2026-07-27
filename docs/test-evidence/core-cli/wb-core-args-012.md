### Case WB-CORE-ARGS-012: Invalid auto read token is preserved for selected validation

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/values.rs > invalid_auto_read_token_is_preserved_for_selected_validation`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `invalid_auto_read_token_is_preserved_for_selected_validation` 直接验证“Invalid auto read token is preserved for selected validation”所描述的结果。
