### Case AUX-RELEASE-CANDIDATE-005: Rejects a target with a non exact public file set

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects a target with a non-exact public file set`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects a target with a non-exact public file set` 直接验证“Rejects a target with a non exact public file set”所描述的结果。
