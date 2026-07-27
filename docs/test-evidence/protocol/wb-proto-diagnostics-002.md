### Case WB-PROTO-DIAGNOSTICS-002: Protocol error codes use diagnostic categories

Entry:
- `crates/shared/protocol/src/tests/basic.rs > protocol_error_codes_use_diagnostic_categories`

Contract:
- `docs/protocol.md` 定义或约束“Protocol diagnostic mapping and projection 保持稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_error_codes_use_diagnostic_categories` 直接验证“Protocol error codes use diagnostic categories”所描述的结果。
