### Case WB-CORE-INVOCATION-LOG-009: Invocation auto read content capture reuses root event and hash shape

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_auto_read_content_capture_reuses_root_event_and_hash_shape`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_auto_read_content_capture_reuses_root_event_and_hash_shape` 直接验证“Invocation auto read content capture reuses root event and hash shape”所描述的结果。
