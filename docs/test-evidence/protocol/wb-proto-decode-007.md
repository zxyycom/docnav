### Case WB-PROTO-DECODE-007: Decode protocol response keeps operation result pairing semantic

Entry:
- `crates/shared/protocol/src/tests/decode.rs > decode_protocol_response_keeps_operation_result_pairing_semantic`

Contract:
- `docs/protocol.md` 定义或约束“Protocol decode wrapper 返回可达阶段结果”所涉及的稳定行为边界。

Proves:
- 原生入口 `decode_protocol_response_keeps_operation_result_pairing_semantic` 直接验证“Decode protocol response keeps operation result pairing semantic”所描述的结果。
