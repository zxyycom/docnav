### Case WB-PROTO-DIAGNOSTICS-004: Protocol error roundtrips through diagnostic record projection

Entry:
- `crates/shared/protocol/src/tests/basic.rs > protocol_error_roundtrips_through_diagnostic_record_projection`

Contract:
- `docs/protocol.md` 定义或约束“Protocol diagnostic mapping and projection 保持稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `protocol_error_roundtrips_through_diagnostic_record_projection` 直接验证“Protocol error roundtrips through diagnostic record projection”所描述的结果。
