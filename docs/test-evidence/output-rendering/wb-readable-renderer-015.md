### Case WB-READABLE-RENDERER-015: Readable error block

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > readable_error_block`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `readable_error_block` 直接验证“Readable error block”所描述的结果。
