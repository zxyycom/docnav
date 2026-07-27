### Case AUX-RELEASE-ARGS-008: Package build target accepts one target option

Entry:
- `scripts/tools/release-package/args.test.ts > package build target accepts one target option`

Contract:
- `docs/testing/release.md` 定义或约束“Release package 参数解析保持边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `package build target accepts one target option` 直接验证“Package build target accepts one target option”所描述的结果。
