### Case AUX-RELEASE-CANDIDATE-006: Rejects workspace version and manifest commit mismatches

Entry:
- `scripts/tools/release-package/candidate.test.ts > rejects workspace version and manifest commit mismatches`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `rejects workspace version and manifest commit mismatches` 直接验证“Rejects workspace version and manifest commit mismatches”所描述的结果。
