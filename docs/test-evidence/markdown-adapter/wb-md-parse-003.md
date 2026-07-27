### Case WB-MD-PARSE-003: Frontmatter is excluded from outline headings

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > frontmatter_is_excluded_from_outline_headings`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown parser 忽略非 heading 结构”所涉及的稳定行为边界。

Proves:
- 原生入口 `frontmatter_is_excluded_from_outline_headings` 直接验证“Frontmatter is excluded from outline headings”所描述的结果。
