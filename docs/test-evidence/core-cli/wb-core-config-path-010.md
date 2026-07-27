### Case WB-CORE-CONFIG-PATH-010: Unsupported config path flag is input error

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > unsupported_config_path_flag_is_input_error`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `unsupported_config_path_flag_is_input_error` 直接验证“Unsupported config path flag is input error”所描述的结果。
