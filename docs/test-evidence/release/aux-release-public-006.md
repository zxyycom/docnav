### Case AUX-RELEASE-PUBLIC-006: A checksum write failure removes public files created after validation

Entry:
- `scripts/tools/release-package/public.test.ts > a checksum write failure removes public files created after validation`

Contract:
- `docs/testing/release.md` 定义或约束“Public files 从已验证 canonical package 派生”所涉及的稳定行为边界。

Proves:
- 原生入口 `a checksum write failure removes public files created after validation` 直接验证“A checksum write failure removes public files created after validation”所描述的结果。
