### Case WB-CORE-PREFLIGHT-004: Document without output defaults to readable view

Entry:
- `crates/docnav/src/cli/preflight/tests.rs > document_without_output_defaults_to_readable_view`

Contract:
- `docs/cli.md` 定义或约束“Core preflight 检测 protocol-json intent”所涉及的稳定行为边界。

Proves:
- 原生入口 `document_without_output_defaults_to_readable_view` 直接验证“Document without output defaults to readable view”所描述的结果。
