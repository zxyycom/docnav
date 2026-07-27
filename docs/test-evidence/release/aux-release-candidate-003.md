### Case AUX-RELEASE-CANDIDATE-003: Accepts only the matching workspace tag and tag commit

Entry:
- `scripts/tools/release-package/candidate.test.ts > accepts only the matching workspace tag and tag commit`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `accepts only the matching workspace tag and tag commit` 直接验证“Accepts only the matching workspace tag and tag commit”所描述的结果。
