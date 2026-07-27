### Case AUX-RELEASE-CANDIDATE-007: Rejects dirty checkout or manifest evidence

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects dirty checkout or manifest evidence`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects dirty checkout or manifest evidence` 直接验证“Rejects dirty checkout or manifest evidence”所描述的结果。
