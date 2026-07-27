### Case AUX-WORKSPACE-VERIFY-018: Reports environment setup errors as failed check results

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > reports environment setup errors as failed check results`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > reports environment setup errors as failed check results` 直接验证“Reports environment setup errors as failed check results”所描述的结果。
