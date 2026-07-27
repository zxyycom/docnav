### Case WB-CLIARGS-BOUNDARY-006: Unused value flag records fact without validating value

Entry:
- `crates/shared/cli-args/src/tests.rs > unused_value_flag_records_fact_without_validating_value`

Contract:
- `docs/cli.md` 定义或约束“Strict CLI 参数扫描保持输入边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `unused_value_flag_records_fact_without_validating_value` 直接验证“Unused value flag records fact without validating value”所描述的结果。
