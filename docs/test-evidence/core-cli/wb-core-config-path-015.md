### Case WB-CORE-CONFIG-PATH-015: Config inspect reports catalog adapter range with exact source

Entry:
- `crates/docnav/src/config/commands/tests.rs > config_inspect_reports_catalog_adapter_range_with_exact_source`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_reports_catalog_adapter_range_with_exact_source` 直接验证“Config inspect reports catalog adapter range with exact source”所描述的结果。
