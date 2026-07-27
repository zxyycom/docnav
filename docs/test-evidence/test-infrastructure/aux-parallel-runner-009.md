### Case AUX-PARALLEL-RUNNER-009: Schedules an explicitly prepared task list

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > schedules an explicitly prepared task list`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > schedules an explicitly prepared task list` 直接验证“Schedules an explicitly prepared task list”所描述的结果。
