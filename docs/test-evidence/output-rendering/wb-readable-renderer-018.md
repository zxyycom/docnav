### Case WB-READABLE-RENDERER-018: Find operation no blocks

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > find_operation_no_blocks`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `find_operation_no_blocks` 直接验证“Find operation no blocks”所描述的结果。
