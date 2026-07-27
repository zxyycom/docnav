### Case WB-MD-ADAPTER-OUTLINE-002: Outline is flat default h1 to h3 and ignores code fences

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > outline_is_flat_default_h1_to_h3_and_ignores_code_fences`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown adapter outline 默认层级和 fallback 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_is_flat_default_h1_to_h3_and_ignores_code_fences` 直接验证“Outline is flat default h1 to h3 and ignores code fences”所描述的结果。
