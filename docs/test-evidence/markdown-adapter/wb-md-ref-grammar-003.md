### Case WB-MD-REF-GRAMMAR-003: Parse canonical heading ref

Entry:
- `crates/adapters/markdown/src/markdown/refs/tests.rs > parse_canonical_heading_ref`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref grammar 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `parse_canonical_heading_ref` 直接验证“Parse canonical heading ref”所描述的结果。
