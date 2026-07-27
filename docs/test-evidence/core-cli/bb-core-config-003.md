### Case BB-CORE-CONFIG-003: Legacy defaults.limit 通过 config source diagnostic 被拒绝

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-CONFIG-003`

Contract:
- `docs/cli.md` 定义或约束“Legacy defaults.limit 通过 config source diagnostic 被拒绝”所涉及的稳定行为边界。

Proves:
- project config 中的 legacy `defaults.limit` 会在真实 `outline` CLI 链路中返回 config-owned `INVALID_REQUEST`。
- structured `unknown_config_field` / `config_issues` diagnostic 报告字段、source level、path origin 和 config path。
