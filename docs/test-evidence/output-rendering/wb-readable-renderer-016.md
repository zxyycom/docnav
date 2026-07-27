### Case WB-READABLE-RENDERER-016: Framing uses lf byte

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > framing_uses_lf_byte`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `framing_uses_lf_byte` 直接验证“Framing uses lf byte”所描述的结果。
