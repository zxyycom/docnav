### Case WB-CORE-CONFIG-PATH-003: Document command parses config file paths as exact values

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > document_command_parses_config_file_paths_as_exact_values`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `document_command_parses_config_file_paths_as_exact_values` 直接验证“Document command parses config file paths as exact values”所描述的结果。
