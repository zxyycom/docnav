### Case WB-CORE-CONFIG-PATH-004: Config inspect parses selected config file paths

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > config_inspect_parses_selected_config_file_paths`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_parses_selected_config_file_paths` 直接验证“Config inspect parses selected config file paths”所描述的结果。
