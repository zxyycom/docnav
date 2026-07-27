### Case AUX-RELEASE-ARGS-009: Package build target rejects extra options and paths

Entry:
- `scripts/tools/release-package/args.test.ts > package build target rejects extra options and paths`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package build target rejects extra options and paths` 直接验证“Package build target rejects extra options and paths”所描述的结果。
