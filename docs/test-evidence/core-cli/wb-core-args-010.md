### Case WB-CORE-ARGS-010: Explicit pagination value is parsed

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/values.rs > explicit_pagination_value_is_parsed`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_pagination_value_is_parsed` 直接验证“Explicit pagination value is parsed”所描述的结果。
