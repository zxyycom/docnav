### Case AUX-SMOKE-HARNESS-004: Records failed task results without stopping other independent tasks

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > records failed task results without stopping other independent tasks`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > records failed task results without stopping other independent tasks` 直接验证“Records failed task results without stopping other independent tasks”所描述的结果。
