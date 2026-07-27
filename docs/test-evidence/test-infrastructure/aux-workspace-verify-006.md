### Case AUX-WORKSPACE-VERIFY-006: Filters catalog success output from docs validator failures

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > filters catalog success output from docs validator failures`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 已知的 test-evidence 与 decision-records 成功输出会被过滤，失败诊断仍保持可见。
