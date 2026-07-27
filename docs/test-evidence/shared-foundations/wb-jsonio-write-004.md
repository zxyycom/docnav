### Case WB-JSONIO-WRITE-004: Serialization failures are distinct from write failures

Entry:
- `crates/shared/json-io/src/tests.rs > serialization_failures_are_distinct_from_write_failures`

Contract:
- `docs/architecture.md` 定义或约束“JSON writer 保持格式和错误分类”所涉及的稳定行为边界。

Proves:
- 原生入口 `serialization_failures_are_distinct_from_write_failures` 直接验证“Serialization failures are distinct from write failures”所描述的结果。
