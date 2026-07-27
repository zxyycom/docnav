### Case AUX-RELEASE-CANDIDATE-009: Rejects canonical package and public hash mismatches

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects canonical package and public hash mismatches`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects canonical package and public hash mismatches` 直接验证“Rejects canonical package and public hash mismatches”所描述的结果。
