### Case AUX-PARALLEL-RUNNER-006: Waits for topological dependencies before starting dependent tasks

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > waits for topological dependencies before starting dependent tasks`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > waits for topological dependencies before starting dependent tasks` 直接验证“Waits for topological dependencies before starting dependent tasks”所描述的结果。
