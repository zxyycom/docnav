### Case AUX-RELEASE-ARGS-003: Package selection accepts a target

Entry:
- `scripts/tools/release-package/args.test.ts > package selection accepts a target`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package selection accepts a target` 直接验证“Package selection accepts a target”所描述的结果。
