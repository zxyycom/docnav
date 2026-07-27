### Case AUX-RELEASE-ARGS-006: Package selection rejects target paths

Entry:
- `scripts/tools/release-package/args.test.ts > package selection rejects target paths`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package selection rejects target paths` 直接验证“Package selection rejects target paths”所描述的结果。
