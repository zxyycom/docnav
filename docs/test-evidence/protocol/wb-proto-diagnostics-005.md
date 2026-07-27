### Case WB-PROTO-DIAGNOSTICS-005: Protocol error location uses config issue path and field

Entry:
- `crates/shared/protocol/src/tests/basic.rs > protocol_error_location_uses_config_issue_path_and_field`

Contract:
- `docs/protocol.md` 定义或约束“Protocol diagnostic mapping and projection 保持稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_error_location_uses_config_issue_path_and_field` 直接验证“Protocol error location uses config issue path and field”所描述的结果。
