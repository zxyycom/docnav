### Case WB-CORE-ARGS-009: Generated page keeps canonical identity for selected validation

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/values.rs > generated_page_keeps_canonical_identity_for_selected_validation`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `generated_page_keeps_canonical_identity_for_selected_validation` 直接验证“Generated page keeps canonical identity for selected validation”所描述的结果。
