### Case WB-CORE-INVOCATION-LOG-007: Invocation read metadata only hashes content without capture file

Entry:
- `crates/docnav/src/runtime/tests/invocation_logging/content.rs > invocation_read_metadata_only_hashes_content_without_capture_file`

Contract:
- `docs/architecture.md` 定义或约束“Core runtime invocation log 保持审计边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `invocation_read_metadata_only_hashes_content_without_capture_file` 直接验证“Invocation read metadata only hashes content without capture file”所描述的结果。
