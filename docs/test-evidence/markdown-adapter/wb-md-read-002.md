### Case WB-MD-READ-002: Read canonical ref resolves matching heading

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > read_canonical_ref_resolves_matching_heading`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown read resolve 和 doc:full ref 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `read_canonical_ref_resolves_matching_heading` 直接验证“Read canonical ref resolves matching heading”所描述的结果。
