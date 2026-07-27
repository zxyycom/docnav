### Case AUX-QUALITY-CACHE-003: Reuses baseline snapshots only when identity and snapshot hash match

Entry:
- `scripts/tools/quality-core/src/measurement/cache.test.ts > quality measurement cache > reuses baseline snapshots only when identity and snapshot hash match`

Contract:
- `docs/tooling.md` 定义或约束“Quality measurement cache identity 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality measurement cache > reuses baseline snapshots only when identity and snapshot hash match` 直接验证“Reuses baseline snapshots only when identity and snapshot hash match”所描述的结果。
