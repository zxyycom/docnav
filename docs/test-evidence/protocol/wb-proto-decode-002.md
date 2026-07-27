### Case WB-PROTO-DECODE-002: Decode protocol request runs contract before raw decode

Entry:
- `crates/shared/protocol/src/tests/decode.rs > decode_protocol_request_runs_contract_before_raw_decode`

Contract:
- `docs/protocol.md` 定义或约束“Protocol decode wrapper 返回可达阶段结果”所涉及的稳定行为边界。

Proves:
- 原生入口 `decode_protocol_request_runs_contract_before_raw_decode` 直接验证“Decode protocol request runs contract before raw decode”所描述的结果。
