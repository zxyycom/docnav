### Case WB-CORE-OUTPUT-005: Document protocol json writes protocol envelope with empty stderr

Entry:
- `crates/docnav/src/output/tests.rs > document_protocol_json_writes_protocol_envelope_with_empty_stderr`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `document_protocol_json_writes_protocol_envelope_with_empty_stderr` 直接验证“Document protocol json writes protocol envelope with empty stderr”所描述的结果。
