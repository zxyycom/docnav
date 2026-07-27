### Case WB-READABLE-RENDERER-020: To readable value serializes valid payload

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > to_readable_value_serializes_valid_payload`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `to_readable_value_serializes_valid_payload` 直接验证“To readable value serializes valid payload”所描述的结果。
