### Case WB-CORE-CONFIG-PATH-018: Init creates and preserves selected project config file

Entry:
- `crates/docnav/src/config/commands/tests.rs > init_creates_and_preserves_selected_project_config_file`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `init_creates_and_preserves_selected_project_config_file` 直接验证“Init creates and preserves selected project config file”所描述的结果。
