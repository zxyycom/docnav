# Claim CLAIM-CLI-MARKDOWN-HEADING-LIMIT-001: Markdown max_heading_level option 通过真实 CLI 生效

Topic: `core-cli`
Owner ref: `docs/adapters/markdown.md#可见性与-max_heading_level`

Statement:
- Markdown max_heading_level controls outline visibility and rejects values outside the adapter-owned boundary.

Observations:
- Markdown `max_heading_level` 可以从 CLI flag 影响 `outline` 可见粒度；越界值作为 adapter-owned option validation error 投影。Project config source 的同类型证明由 `CLAIM-CLI-ADAPTER-CONFIG-APPLICABILITY-001` 承担。

Supported by:
- `smoke|core:real-markdown-link-chain|CORE-MD-OPTIONS-001`
- `smoke|core:real-markdown-link-chain|CORE-MD-OPTIONS-002`
