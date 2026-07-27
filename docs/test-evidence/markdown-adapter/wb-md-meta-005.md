### Case WB-MD-META-005: Info returns markdown summary

Entry:
- `crates/adapters/markdown/tests/adapter/meta.rs > info_returns_markdown_summary`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown manifest/probe/info 元数据稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `info_returns_markdown_summary` 直接验证“Info returns markdown summary”所描述的结果。
