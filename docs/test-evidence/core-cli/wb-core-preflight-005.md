### Case WB-CORE-PREFLIGHT-005: Non document output context keeps plain command semantics

Entry:
- `crates/docnav/src/cli/preflight/tests.rs > non_document_output_context_keeps_plain_command_semantics`

Contract:
- `docs/cli.md` 定义或约束“Core preflight 检测 protocol-json intent”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_document_output_context_keeps_plain_command_semantics` 直接验证“Non document output context keeps plain command semantics”所描述的结果。
