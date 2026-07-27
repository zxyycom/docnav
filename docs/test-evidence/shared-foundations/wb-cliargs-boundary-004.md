### Case WB-CLIARGS-BOUNDARY-004: Used value flag requires a value before known value flag

Entry:
- `crates/shared/cli-args/src/tests.rs > used_value_flag_requires_a_value_before_known_value_flag`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `used_value_flag_requires_a_value_before_known_value_flag` 直接验证“Used value flag requires a value before known value flag”所描述的结果。
