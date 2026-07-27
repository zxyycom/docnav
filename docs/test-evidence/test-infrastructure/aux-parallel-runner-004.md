### Case AUX-PARALLEL-RUNNER-004: Does not limit concurrency when no explicit concurrency is provided

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > does not limit concurrency when no explicit concurrency is provided`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > does not limit concurrency when no explicit concurrency is provided` 直接验证“Does not limit concurrency when no explicit concurrency is provided”所描述的结果。
