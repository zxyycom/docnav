### Case WB-CORE-CONFIG-PATH-009: Inline config path value can start with known flag text

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > inline_config_path_value_can_start_with_known_flag_text`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `inline_config_path_value_can_start_with_known_flag_text` 直接验证“Inline config path value can start with known flag text”所描述的结果。
