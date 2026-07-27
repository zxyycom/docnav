### Case AUX-WORKSPACE-VERIFY-012: Parses verification profile arguments

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > parses verification profile arguments`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > parses verification profile arguments` 直接验证“Parses verification profile arguments”所描述的结果。
