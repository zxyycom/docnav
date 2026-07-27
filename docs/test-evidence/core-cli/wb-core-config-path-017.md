### Case WB-CORE-CONFIG-PATH-017: Config inspect serializes complete invalid json load status

Entry:
- `crates/docnav/src/config/commands/tests.rs > config_inspect_serializes_complete_invalid_json_load_status`

Contract:
- `docs/cli.md` 定义或约束“Core config inspect uses selected config file paths”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_serializes_complete_invalid_json_load_status` 直接验证“Config inspect serializes complete invalid json load status”所描述的结果。
