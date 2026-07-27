### Case WB-CORE-HELP-006: Version command has no output mode

Entry:
- `crates/docnav/src/cli/parser/tests/help.rs > version_command_has_no_output_mode`

Contract:
- `docs/cli.md` 定义或约束“Core parser help/version 不进入 document output mode”所涉及的稳定行为边界。

Proves:
- 原生入口 `version_command_has_no_output_mode` 直接验证“Version command has no output mode”所描述的结果。
