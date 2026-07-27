### Case AUX-SMOKE-HARNESS-003: Runs independent smoke tasks concurrently and keeps per task command counts isolated

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > runs independent smoke tasks concurrently and keeps per-task command counts isolated`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > runs independent smoke tasks concurrently and keeps per-task command counts isolated` 直接验证“Runs independent smoke tasks concurrently and keeps per task command counts isolated”所描述的结果。
