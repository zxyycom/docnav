### Case AUX-RELEASE-CANDIDATE-002: Accepts an exact manual run candidate without modifying its files

Entry:
- `scripts/tools/release-package/candidate.test.ts > accepts an exact manual-run candidate without modifying its files`

Contract:
- `docs/testing/release.md` 定义或约束“Release candidate 聚合证据保持同源”所涉及的稳定行为边界。

Proves:
- 原生入口 `accepts an exact manual-run candidate without modifying its files` 直接验证“Accepts an exact manual run candidate without modifying its files”所描述的结果。
