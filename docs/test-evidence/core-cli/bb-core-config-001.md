### Case BB-CORE-CONFIG-001: Config inspect source status 与参数事实可观察

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-CONFIG-001`

Contract:
- `docs/cli.md` 定义或约束“Config inspect source status 与参数事实可观察”所涉及的稳定行为边界。

Proves:
- `docnav config inspect` reports selected project/user source scope、origin、load state、source diagnostics and current adapter/output/pagination parameter facts without modifying either selected file。
- Inspect output includes the config-source projection entry for the observed pagination field.
- Disabled pagination configured through direct config file edit remains observable through inspect facts；an `outline` command with an explicit numeric `--limit` returns the complete three-entry fixture outline with `page: null`.
