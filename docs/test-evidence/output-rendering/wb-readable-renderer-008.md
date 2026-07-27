### Case WB-READABLE-RENDERER-008: Crlf payload preserved in block

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > crlf_payload_preserved_in_block`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `crlf_payload_preserved_in_block` 直接验证“Crlf payload preserved in block”所描述的结果。
