### Case WB-MD-REF-GRAMMAR-002: Canonical heading ref uses structural coordinates

Entry:
- `crates/adapters/markdown/src/markdown/refs/tests.rs > canonical_heading_ref_uses_structural_coordinates`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown ref grammar 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `canonical_heading_ref_uses_structural_coordinates` 直接验证“Canonical heading ref uses structural coordinates”所描述的结果。
