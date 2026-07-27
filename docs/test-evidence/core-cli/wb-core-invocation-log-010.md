### Case WB-CORE-INVOCATION-LOG-010: Invocation find auto read logs root metadata without capture file

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_find_auto_read_logs_root_metadata_without_capture_file`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_find_auto_read_logs_root_metadata_without_capture_file` 直接验证“Invocation find auto read logs root metadata without capture file”所描述的结果。
