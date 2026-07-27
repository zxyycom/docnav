### Case WB-CORE-ARGS-013: Explicit max heading level value is parsed for supported operations

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/values.rs > explicit_max_heading_level_value_is_parsed_for_supported_operations`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_max_heading_level_value_is_parsed_for_supported_operations` 直接验证“Explicit max heading level value is parsed for supported operations”所描述的结果。
