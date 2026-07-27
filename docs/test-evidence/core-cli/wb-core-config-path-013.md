### Case WB-CORE-CONFIG-PATH-013: Config inspect omits optional non json null parameter fact

Entry:
- `crates/docnav/src/config/commands/tests.rs > config_inspect_omits_optional_non_json_null_parameter_fact`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_omits_optional_non_json_null_parameter_fact` 直接验证“Config inspect omits optional non json null parameter fact”所描述的结果。
