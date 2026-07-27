### Case AUX-PARALLEL-RUNNER-003: Runs independent tasks concurrently but serializes matching mutexes

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > runs independent tasks concurrently but serializes matching mutexes`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > runs independent tasks concurrently but serializes matching mutexes` 直接验证“Runs independent tasks concurrently but serializes matching mutexes”所描述的结果。
