### Case AUX-SMOKE-HARNESS-005: Runs nested case tasks but records only the parent smoke report

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > runs nested case tasks but records only the parent smoke report`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > runs nested case tasks but records only the parent smoke report` 直接验证“Runs nested case tasks but records only the parent smoke report”所描述的结果。
