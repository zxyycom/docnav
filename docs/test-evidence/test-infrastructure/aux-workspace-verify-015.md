### Case AUX-WORKSPACE-VERIFY-015: Removes copied development binary artifacts

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > removes copied development binary artifacts`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > removes copied development binary artifacts` 直接验证“Removes copied development binary artifacts”所描述的结果。
