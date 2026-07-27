### Case WB-OUTPUT-DOCUMENT-002: Custom renderer receives success response and controls exact text

Entry:
- `crates/shared/output/src/tests.rs > custom_renderer_receives_success_response_and_controls_exact_text`

Contract:
- `docs/output.md` 定义或约束“共享 document output facade 分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `custom_renderer_receives_success_response_and_controls_exact_text` 直接验证“Custom renderer receives success response and controls exact text”所描述的结果。
