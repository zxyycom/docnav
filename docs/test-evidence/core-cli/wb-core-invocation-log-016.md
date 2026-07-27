### Case WB-CORE-INVOCATION-LOG-016: Invocation logging enabled success writes jsonl with request id

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/output.rs > invocation_logging_enabled_success_writes_jsonl_with_request_id`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_logging_enabled_success_writes_jsonl_with_request_id` 直接验证“Invocation logging enabled success writes jsonl with request id”所描述的结果。
