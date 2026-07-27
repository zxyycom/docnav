### Case AUX-PARALLEL-RUNNER-010: Rejects duplicate ids and unknown dependencies

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > rejects duplicate ids and unknown dependencies`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > rejects duplicate ids and unknown dependencies` 直接验证“Rejects duplicate ids and unknown dependencies”所描述的结果。
