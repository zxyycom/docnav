### Case WB-CLIARGS-BOUNDARY-005: Used value flag allows unknown hyphen value

Entry:
- `crates/shared/cli-args/src/tests.rs > used_value_flag_allows_unknown_hyphen_value`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `used_value_flag_allows_unknown_hyphen_value` 直接验证“Used value flag allows unknown hyphen value”所描述的结果。
