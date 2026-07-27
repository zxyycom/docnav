### Case WB-READABLE-RENDERER-005: Utf8 byte length is correct

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > utf8_byte_length_is_correct`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `utf8_byte_length_is_correct` 直接验证“Utf8 byte length is correct”所描述的结果。
