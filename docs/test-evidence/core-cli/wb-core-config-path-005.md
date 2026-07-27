### Case WB-CORE-CONFIG-PATH-005: Legacy config subcommands are rejected

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > legacy_config_subcommands_are_rejected`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `legacy_config_subcommands_are_rejected` 直接验证“Legacy config subcommands are rejected”所描述的结果。
