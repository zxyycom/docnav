### Case WB-CORE-INVOCATION-LOG-008: Invocation content capture writes hash named file and event

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_content_capture_writes_hash_named_file_and_event`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_content_capture_writes_hash_named_file_and_event` 直接验证“Invocation content capture writes hash named file and event”所描述的结果。
