### Case WB-CORE-PREFLIGHT-002: Detects space separated protocol json output

Entry:
- `crates/docnav/src/cli/preflight/tests.rs > detects_space_separated_protocol_json_output`

Contract:
- `docs/cli.md` 定义或约束“Core preflight 检测 protocol-json intent”所涉及的稳定行为边界。

Proves:
- 原生入口 `detects_space_separated_protocol_json_output` 直接验证“Detects space separated protocol json output”所描述的结果。
