### Case WB-READABLE-RENDERER-012: Multiple blocks with nested pointer

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > multiple_blocks_with_nested_pointer`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `multiple_blocks_with_nested_pointer` 直接验证“Multiple blocks with nested pointer”所描述的结果。
