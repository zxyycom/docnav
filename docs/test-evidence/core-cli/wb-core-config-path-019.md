### Case WB-CORE-CONFIG-PATH-019: Init rejects selected project config directory

Entry:
- `crates/docnav/src/config/commands/tests.rs > init_rejects_selected_project_config_directory`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `init_rejects_selected_project_config_directory` 直接验证“Init rejects selected project config directory”所描述的结果。
