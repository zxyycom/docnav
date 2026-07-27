### Case WB-CLIARGS-BOUNDARY-003: Used value flag is retained and consumes value

Entry:
- `crates/shared/cli-args/src/tests.rs > used_value_flag_is_retained_and_consumes_value`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `used_value_flag_is_retained_and_consumes_value` 直接验证“Used value flag is retained and consumes value”所描述的结果。
