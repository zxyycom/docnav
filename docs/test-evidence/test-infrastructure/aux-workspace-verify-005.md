### Case AUX-WORKSPACE-VERIFY-005: Suppresses all passed output even when a success line is not configured

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > suppresses all passed output even when a success line is not configured`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > suppresses all passed output even when a success line is not configured` 直接验证“Suppresses all passed output even when a success line is not configured”所描述的结果。
