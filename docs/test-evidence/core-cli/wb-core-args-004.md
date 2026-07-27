### Case WB-CORE-ARGS-004: Generated value flag without value maps clap structural error

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/structural_errors.rs > generated_value_flag_without_value_maps_clap_structural_error`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `generated_value_flag_without_value_maps_clap_structural_error` 直接验证“Generated value flag without value maps clap structural error”所描述的结果。
