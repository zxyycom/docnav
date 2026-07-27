### Case AUX-RELEASE-WORKFLOW-005: Publish is the single writer and creates one new prerelease from four public files

Entry:
- `scripts/tools/release-package/workflow.test.ts > publish is the single writer and creates one new prerelease from four public files`

Contract:
- `docs/testing/release.md` 定义或约束“Beta release workflow 保持验证与 promotion 门禁”所涉及的稳定行为边界。

Proves:
- 原生入口 `publish is the single writer and creates one new prerelease from four public files` 直接验证“Publish is the single writer and creates one new prerelease from four public files”所描述的结果。
