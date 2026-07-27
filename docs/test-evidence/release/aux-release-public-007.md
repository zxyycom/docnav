### Case AUX-RELEASE-PUBLIC-007: A missing manifest does not remove an unrelated public directory

Entry:
- `scripts/tools/release-package/public.test.ts > a missing manifest does not remove an unrelated public directory`

Contract:
- `docs/testing/release.md` 定义或约束“Public files 从已验证 canonical package 派生”所涉及的稳定行为边界。

Proves:
- 原生入口 `a missing manifest does not remove an unrelated public directory` 直接验证“A missing manifest does not remove an unrelated public directory”所描述的结果。
