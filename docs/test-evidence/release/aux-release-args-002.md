### Case AUX-RELEASE-ARGS-002: Package selection defaults to the current host package

Entry:
- `scripts/tools/release-package/args.test.ts > package selection defaults to the current host package`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package selection defaults to the current host package` 直接验证“Package selection defaults to the current host package”所描述的结果。
