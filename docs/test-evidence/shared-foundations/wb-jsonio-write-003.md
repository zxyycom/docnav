### Case WB-JSONIO-WRITE-003: Pretty json writes value and newline

Entry:
- `crates/shared/json-io/src/tests.rs > pretty_json_writes_value_and_newline`

Contract:
- `docs/architecture.md` 定义或约束“JSON writer 保持格式和错误分类”所涉及的稳定行为边界。

Proves:
- 原生入口 `pretty_json_writes_value_and_newline` 直接验证“Pretty json writes value and newline”所描述的结果。
