### Case WB-READABLE-RENDERER-007: Combined character utf8 byte length

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > combined_character_utf8_byte_length`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `combined_character_utf8_byte_length` 直接验证“Combined character utf8 byte length”所描述的结果。
