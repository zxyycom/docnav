### Case AUX-WORKSPACE-VERIFY-019: Rejects invalid leaf and group check definitions

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > rejects invalid leaf and group check definitions`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > rejects invalid leaf and group check definitions` 直接验证“Rejects invalid leaf and group check definitions”所描述的结果。
