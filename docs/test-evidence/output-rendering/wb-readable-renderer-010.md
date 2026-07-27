### Case WB-READABLE-RENDERER-010: Trailing lf payload no extra framing lf

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > trailing_lf_payload_no_extra_framing_lf`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `trailing_lf_payload_no_extra_framing_lf` 直接验证“Trailing lf payload no extra framing lf”所描述的结果。
