# Claim CLAIM-CLI-ADAPTER-CONFIG-APPLICABILITY-001: Adapter-scoped config 按 catalog operation applicability 生效

Topic: `core-cli`
Owner ref: `docs/navigation-input-resolution.md#selected-operation-catalog-view`

Statement:
- Adapter-scoped config affects an operation only through the selected adapter and operation catalog view.

Observations:
- Project config 中的 `options.docnav-markdown.max_heading_level` 通过 core-authored Markdown-scoped catalog entry 影响 `outline` entries。
- User config 中的 `options.docnav-markdown.max_heading_level` 通过 direct config file edit/read 参与 source priority；当 catalog 不把该参数绑定到 selected operation 时，返回 structured unsupported diagnostic 并保留 source level/path。

Supported by:
- `smoke|core:config-context|CORE-CONFIG-004`
