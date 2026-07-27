### Case WB-CORE-CONFIG-PATH-012: Config inspect reports selected sources and parameter facts without writing

Entry:
- `crates/docnav/src/config/commands/tests.rs > config_inspect_reports_selected_sources_and_parameter_facts_without_writing`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_reports_selected_sources_and_parameter_facts_without_writing` 直接验证“Config inspect reports selected sources and parameter facts without writing”所描述的结果。
