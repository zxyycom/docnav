### Case WB-CORE-CONFIG-PATH-008: Config path flag before known flag is missing value input error

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > config_path_flag_before_known_flag_is_missing_value_input_error`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_path_flag_before_known_flag_is_missing_value_input_error` 直接验证“Config path flag before known flag is missing value input error”所描述的结果。
