### Case BB-CORE-MD-OPTIONS-001: Markdown max_heading_level option 通过真实 CLI 生效

Entry:
- `test/smoke/core/cases/real-markdown.ts > smoke task CORE-MD-OPTIONS-001`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown max_heading_level option 通过真实 CLI 生效”所涉及的稳定行为边界。

Proves:
- Markdown `max_heading_level` 可以从 CLI flag 影响 `outline` 可见粒度；越界值作为 adapter-owned option validation error 投影。Project config source 的同类型证明由 `BB-CORE-CONFIG-004` 承担。
