### Case AUX-RELEASE-WORKFLOW-004: Aggregate validation consumes current run artifacts for manual and tag inputs

Entry:
- `scripts/tools/release-package/workflow.test.ts > aggregate validation consumes current-run artifacts for manual and tag inputs`

Contract:
- `docs/testing/release.md` 定义或约束“Beta release workflow 保持验证与 promotion 门禁”所涉及的稳定行为边界。

Proves:
- 原生入口 `aggregate validation consumes current-run artifacts for manual and tag inputs` 直接验证“Aggregate validation consumes current run artifacts for manual and tag inputs”所描述的结果。
