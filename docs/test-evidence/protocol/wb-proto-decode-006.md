### Case WB-PROTO-DECODE-006: Decode probe result returns semantic error with typed value

Entry:
- `crates/shared/protocol/src/tests/decode.rs > decode_probe_result_returns_semantic_error_with_typed_value`

Contract:
- `docs/protocol.md` 定义或约束“Protocol decode wrapper 返回可达阶段结果”所涉及的稳定行为边界。

Proves:
- 原生入口 `decode_probe_result_returns_semantic_error_with_typed_value` 直接验证“Decode probe result returns semantic error with typed value”所描述的结果。
