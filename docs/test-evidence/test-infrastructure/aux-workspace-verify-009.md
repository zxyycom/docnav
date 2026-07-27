### Case AUX-WORKSPACE-VERIFY-009: Filters cargo trybuild success noise from successful cargo test output

Entry:
- `scripts/docnav-workspace/verify.test.ts > workspace verifier configuration > filters cargo trybuild success noise from successful cargo test output`

Contract:
- `docs/testing.md` 定义或约束“Workspace verifier 保持 required/full profile 语义”所涉及的稳定行为边界。

Proves:
- 原生入口 `workspace verifier configuration > filters cargo trybuild success noise from successful cargo test output` 直接验证“Filters cargo trybuild success noise from successful cargo test output”所描述的结果。
