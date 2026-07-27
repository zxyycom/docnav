### Case AUX-RELEASE-ARGS-005: Package selection rejects ambiguous selectors

Entry:
- `scripts/tools/release-package/args.test.ts > package selection rejects ambiguous selectors`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package selection rejects ambiguous selectors` 直接验证“Package selection rejects ambiguous selectors”所描述的结果。
