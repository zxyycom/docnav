### Case WB-READABLE-RENDERER-003: Outline no blocks emits header only

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > outline_no_blocks_emits_header_only`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_no_blocks_emits_header_only` 直接验证“Outline no blocks emits header only”所描述的结果。
