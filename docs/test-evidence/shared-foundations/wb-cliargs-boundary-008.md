### Case WB-CLIARGS-BOUNDARY-008: Switch flags are retained without consuming value

Entry:
- `crates/shared/cli-args/src/tests.rs > switch_flags_are_retained_without_consuming_value`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `switch_flags_are_retained_without_consuming_value` 直接验证“Switch flags are retained without consuming value”所描述的结果。
