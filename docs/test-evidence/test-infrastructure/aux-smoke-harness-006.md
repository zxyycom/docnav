### Case AUX-SMOKE-HARNESS-006: Uses DOCNAV SMOKE CONCURRENCY at the smoke scheduling boundary

Entry:
- `test/tools/smoke-harness.test.ts > smoke harness task scheduling > uses DOCNAV_SMOKE_CONCURRENCY at the smoke scheduling boundary`

Contract:
- `docs/testing.md` 定义或约束“Smoke harness 正确记录 task 和 command 输出语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `smoke harness task scheduling > uses DOCNAV_SMOKE_CONCURRENCY at the smoke scheduling boundary` 直接验证“Uses DOCNAV SMOKE CONCURRENCY at the smoke scheduling boundary”所描述的结果。
