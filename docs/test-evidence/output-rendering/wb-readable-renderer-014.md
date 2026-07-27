### Case WB-READABLE-RENDERER-014: Undeclared fields preserved in header

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > undeclared_fields_preserved_in_header`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `undeclared_fields_preserved_in_header` 直接验证“Undeclared fields preserved in header”所描述的结果。
