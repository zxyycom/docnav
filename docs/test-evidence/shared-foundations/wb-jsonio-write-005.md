### Case WB-JSONIO-WRITE-005: Write failures are reported

Entry:
- `crates/shared/json-io/src/tests.rs > write_failures_are_reported`

Contract:
- `docs/architecture.md` 定义或约束“JSON writer 保持格式和错误分类”所涉及的稳定行为边界。

Proves:
- 原生入口 `write_failures_are_reported` 直接验证“Write failures are reported”所描述的结果。
