### Case WB-CORE-CONFIG-PATH-011: Init rejects user config path flag

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > init_rejects_user_config_path_flag`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `init_rejects_user_config_path_flag` 直接验证“Init rejects user config path flag”所描述的结果。
