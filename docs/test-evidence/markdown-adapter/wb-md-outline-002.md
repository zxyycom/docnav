### Case WB-MD-OUTLINE-002: Outline generates canonical heading refs

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > outline_generates_canonical_heading_refs`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline ref 和 display 语义稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_generates_canonical_heading_refs` 直接验证“Outline generates canonical heading refs”所描述的结果。
