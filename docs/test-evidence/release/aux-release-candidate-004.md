### Case AUX-RELEASE-CANDIDATE-004: Rejects a candidate with a non exact direct target set

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects a candidate with a non-exact direct target set`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects a candidate with a non-exact direct target set` 直接验证“Rejects a candidate with a non exact direct target set”所描述的结果。
