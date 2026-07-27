### Case WB-READABLE-RENDERER-011: Payload contains block marker text

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > payload_contains_block_marker_text`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `payload_contains_block_marker_text` 直接验证“Payload contains block marker text”所描述的结果。
