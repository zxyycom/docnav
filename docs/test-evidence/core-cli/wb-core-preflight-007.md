### Case WB-CORE-PREFLIGHT-007: Legacy config failure uses protocol json framing

Entry:
- `crates/docnav/src/cli/preflight/tests.rs > legacy_config_failure_uses_protocol_json_framing`

Contract:
- `docs/cli.md` 定义或约束“Core preflight 检测 protocol-json intent”所涉及的稳定行为边界。

Proves:
- 原生入口 `legacy_config_failure_uses_protocol_json_framing` 直接验证“Legacy config failure uses protocol json framing”所描述的结果。
