### Case WB-CORE-HELP-004: Help text scopes catalog parameters to supported operations

Entry:
- `crates/docnav/src/cli/parser/tests/help.rs > help_text_scopes_catalog_parameters_to_supported_operations`

Contract:
- `docs/cli.md` 定义或约束“Core parser help/version 不进入 document output mode”所涉及的稳定行为边界。

Proves:
- 原生入口 `help_text_scopes_catalog_parameters_to_supported_operations` 直接验证“Help text scopes catalog parameters to supported operations”所描述的结果。
