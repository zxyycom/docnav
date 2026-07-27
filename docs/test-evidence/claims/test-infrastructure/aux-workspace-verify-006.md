# Claim CLAIM-AUX-WORKSPACE-VERIFY-006: Filters catalog success output from docs validator failures

Topic: `test-infrastructure`
Owner ref: `docs/testing.md#统一验证入口`

Statement:
- Workspace verification suppresses known successful validator noise while preserving actionable failure diagnostics.

Observations:
- 已知的 test-evidence 与 decision-records 成功输出会被过滤，失败诊断仍保持可见。

Supported by:
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters catalog success output from docs validator failures`
