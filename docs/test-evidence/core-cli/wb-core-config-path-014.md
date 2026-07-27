### Case WB-CORE-CONFIG-PATH-014: Config inspect reports validation diagnostics without failing

Entry:
- `crates/docnav/src/config/commands/tests.rs > config_inspect_reports_validation_diagnostics_without_failing`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_reports_validation_diagnostics_without_failing` 直接验证“Config inspect reports validation diagnostics without failing”所描述的结果。
