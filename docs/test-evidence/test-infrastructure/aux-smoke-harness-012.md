### Case AUX-SMOKE-HARNESS-012: Copies config fixtures before mutable config cases write

Entry:
- `test/smoke/core/fixtures/project.test.ts > core smoke fixture projects > copies config fixtures before mutable config cases write`

Contract:
- `docs/testing.md` 定义或约束“Core smoke config fixture helper 保持配置/文档分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `core smoke fixture projects > copies config fixtures before mutable config cases write` 直接验证“Copies config fixtures before mutable config cases write”所描述的结果。
