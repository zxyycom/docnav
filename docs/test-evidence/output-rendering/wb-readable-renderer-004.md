### Case WB-READABLE-RENDERER-004: Read content block

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > read_content_block`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_content_block` 直接验证“Read content block”所描述的结果。
