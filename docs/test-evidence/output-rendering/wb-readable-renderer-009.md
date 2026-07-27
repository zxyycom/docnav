### Case WB-READABLE-RENDERER-009: No trailing lf payload gets framing lf

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > no_trailing_lf_payload_gets_framing_lf`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `no_trailing_lf_payload_gets_framing_lf` 直接验证“No trailing lf payload gets framing lf”所描述的结果。
