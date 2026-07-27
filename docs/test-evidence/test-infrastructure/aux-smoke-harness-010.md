### Case AUX-SMOKE-HARNESS-010: Creates and cleans only the owned core smoke run directory after task failure

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > creates and cleans only the owned core smoke run directory after task failure`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > creates and cleans only the owned core smoke run directory after task failure` 直接验证“Creates and cleans only the owned core smoke run directory after task failure”所描述的结果。
