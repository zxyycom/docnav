### Case WB-MD-PARSE-002: Parser ignores code fence pseudo heading and invalid heading

Entry:
- `crates/adapters/markdown/src/markdown/tests.rs > parser_ignores_code_fence_pseudo_heading_and_invalid_heading`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown parser 忽略非 heading 结构”所涉及的稳定行为边界。

Proves:
- 原生入口 `parser_ignores_code_fence_pseudo_heading_and_invalid_heading` 直接验证“Parser ignores code fence pseudo heading and invalid heading”所描述的结果。
