### Case WB-CORE-HELP-002: Help returns typed help command

Entry:
- `crates/docnav/src/cli/parser/tests/help.rs > help_returns_typed_help_command`

Contract:
- `docs/cli.md` 定义或约束“Core parser help/version 不进入 document output mode”所涉及的稳定行为边界。

Proves:
- 原生入口 `help_returns_typed_help_command` 直接验证“Help returns typed help command”所描述的结果。
