### Case AUX-PARALLEL-RUNNER-011: Rejects invalid task list metadata at the normalization boundary

Entry:
- `scripts/tools/parallel-task-runner/test/index.test.ts > parallel task runner > rejects invalid task list metadata at the normalization boundary`

Contract:
- `docs/testing.md` 定义或约束“Parallel task runner 保持调度契约”所涉及的稳定行为边界。

Proves:
- 原生入口 `parallel task runner > rejects invalid task list metadata at the normalization boundary` 直接验证“Rejects invalid task list metadata at the normalization boundary”所描述的结果。
