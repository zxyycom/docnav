### Case AUX-PARALLEL-RUNNER-007: Waits for onComplete while treating resolved result values as opaque

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > waits for onComplete while treating resolved result values as opaque`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > waits for onComplete while treating resolved result values as opaque` 直接验证“Waits for onComplete while treating resolved result values as opaque”所描述的结果。
