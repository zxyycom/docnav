### Case WB-PROTO-DECODE-005: Decode manifest returns the typed current manifest

Entry:
- `crates/shared/protocol/src/tests/decode.rs > decode_manifest_returns_the_typed_current_manifest`

Contract:
- `docs/protocol.md` 定义或约束“Protocol decode wrapper 返回可达阶段结果”所涉及的稳定行为边界。

Proves:
- 原生入口 `decode_manifest_returns_the_typed_current_manifest` 直接验证“Decode manifest returns the typed current manifest”所描述的结果。
