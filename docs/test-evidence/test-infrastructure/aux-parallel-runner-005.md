### Case AUX-PARALLEL-RUNNER-005: Respects an explicit concurrency limit

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > respects an explicit concurrency limit`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > respects an explicit concurrency limit` 直接验证“Respects an explicit concurrency limit”所描述的结果。
