### Case AUX-QUALITY-SCAN-CLI-002: Skips baseline by default and keeps baseline generation opt in

Entry:
- `scripts/quality/args.test.ts > quality scan CLI args > skips baseline by default and keeps baseline generation opt-in`

Contract:
- `docs/tooling.md` 定义或约束“Quality scan CLI 默认值稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality scan CLI args > skips baseline by default and keeps baseline generation opt-in` 直接验证“Skips baseline by default and keeps baseline generation opt in”所描述的结果。
