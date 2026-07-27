### Case WB-READABLE-RENDERER-013: Empty string block zero bytes

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > empty_string_block_zero_bytes`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `empty_string_block_zero_bytes` 直接验证“Empty string block zero bytes”所描述的结果。
