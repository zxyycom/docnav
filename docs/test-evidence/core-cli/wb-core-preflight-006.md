### Case WB-CORE-PREFLIGHT-006: Non document protocol json hint uses core output flag

Entry:
- `crates/docnav/src/cli/preflight/tests.rs > non_document_protocol_json_hint_uses_core_output_flag`

Contract:
- `docs/cli.md` 定义或约束“Core preflight 检测 protocol-json intent”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_document_protocol_json_hint_uses_core_output_flag` 直接验证“Non document protocol json hint uses core output flag”所描述的结果。
