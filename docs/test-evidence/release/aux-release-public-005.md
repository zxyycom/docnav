### Case AUX-RELEASE-PUBLIC-005: Mismatched canonical package evidence fails without modifying an existing public set

Entry:
- `scripts/tools/release-package/public.test.ts > mismatched canonical package evidence fails without modifying an existing public set`

Contract:
- `docs/testing/release.md` 定义或约束“Public files 从已验证 canonical package 派生”所涉及的稳定行为边界。

Proves:
- 原生入口 `mismatched canonical package evidence fails without modifying an existing public set` 直接验证“Mismatched canonical package evidence fails without modifying an existing public set”所描述的结果。
