### Case WB-OUTPUT-DOCUMENT-003: Custom renderer receives failure response

Entry:
- `crates/shared/output/src/tests.rs > custom_renderer_receives_failure_response`

Contract:
- `docs/output.md` 定义或约束“共享 document output facade 分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `custom_renderer_receives_failure_response` 直接验证“Custom renderer receives failure response”所描述的结果。
