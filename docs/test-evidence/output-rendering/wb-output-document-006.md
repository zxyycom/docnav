### Case WB-OUTPUT-DOCUMENT-006: Protocol json serializes success and failure responses without rendering

Entry:
- `crates/shared/output/src/tests.rs > protocol_json_serializes_success_and_failure_responses_without_rendering`

Contract:
- `docs/output.md` 定义或约束“共享 document output facade 分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_json_serializes_success_and_failure_responses_without_rendering` 直接验证“Protocol json serializes success and failure responses without rendering”所描述的结果。
