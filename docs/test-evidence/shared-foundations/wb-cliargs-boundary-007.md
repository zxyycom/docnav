### Case WB-CLIARGS-BOUNDARY-007: Unused value flag requires a value

Entry:
- `crates/shared/cli-args/src/tests.rs > unused_value_flag_requires_a_value`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `unused_value_flag_requires_a_value` 直接验证“Unused value flag requires a value”所描述的结果。
