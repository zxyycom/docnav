### Case WB-PROTO-DECODE-004: Decode protocol request preserves defaultable arguments

Entry:
- `crates/shared/protocol/src/tests/decode.rs > decode_protocol_request_preserves_defaultable_arguments`

Contract:
- `docs/protocol.md` 定义或约束“Protocol decode wrapper 返回可达阶段结果”所涉及的稳定行为边界。

Proves:
- 原生入口 `decode_protocol_request_preserves_defaultable_arguments` 直接验证“Decode protocol request preserves defaultable arguments”所描述的结果。
