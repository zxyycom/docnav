### Case AUX-RELEASE-CANDIDATE-008: Rejects package evidence from a different workflow run

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects package evidence from a different workflow run`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects package evidence from a different workflow run` 直接验证“Rejects package evidence from a different workflow run”所描述的结果。
