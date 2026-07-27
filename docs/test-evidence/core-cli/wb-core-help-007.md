### Case WB-CORE-HELP-007: Non document surfaces keep their own command shapes

Entry:
- `crates/docnav/src/cli/parser/tests/help.rs > non_document_surfaces_keep_their_own_command_shapes`

Contract:
- `docs/cli.md` 定义或约束“Core parser help/version 不进入 document output mode”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_document_surfaces_keep_their_own_command_shapes` 直接验证“Non document surfaces keep their own command shapes”所描述的结果。
