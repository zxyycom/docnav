### Case WB-CORE-HELP-003: Help text shows only public output modes

Entry:
- `crates/docnav/src/cli/parser/tests/help.rs > help_text_shows_only_public_output_modes`

Contract:
- `docs/cli.md` 定义或约束“Core parser help/version 不进入 document output mode”所涉及的稳定行为边界。

Proves:
- 原生入口 `help_text_shows_only_public_output_modes` 直接验证“Help text shows only public output modes”所描述的结果。
