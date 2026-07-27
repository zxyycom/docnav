### Case AUX-SMOKE-HARNESS-009: Validates smoke concurrency values

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > validates smoke concurrency values`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > validates smoke concurrency values` 直接验证“Validates smoke concurrency values”所描述的结果。
