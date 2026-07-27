### Case BB-CORE-CONFIG-004: Adapter-scoped config 按 catalog operation applicability 生效

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-CONFIG-004`

Contract:
- `docs/cli.md` 定义或约束“Adapter-scoped config 按 catalog operation applicability 生效”所涉及的稳定行为边界。

Proves:
- Project config 中的 `options.docnav-markdown.max_heading_level` 通过 core-authored Markdown-scoped catalog entry 影响 `outline` entries。
- User config 中的 `options.docnav-markdown.max_heading_level` 通过 direct config file edit/read 参与 source priority；当 catalog 不把该参数绑定到 selected operation 时，返回 structured unsupported diagnostic 并保留 source level/path。
