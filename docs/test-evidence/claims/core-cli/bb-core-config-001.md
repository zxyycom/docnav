# Claim CLAIM-BB-CORE-CONFIG-001: Config inspect source status 与参数事实可观察

Topic: `core-cli`
Owner ref: `docs/cli.md#配置命令`

Statement:
- Config inspection is read-only and exposes selected source identity, load state, diagnostics and current parameter facts.

Observations:
- `docnav config inspect` reports selected project/user source scope、origin、load state、source diagnostics and current adapter/output/pagination parameter facts without modifying either selected file。
- Inspect output includes the config-source projection entry for the observed pagination field.
- Disabled pagination configured through direct config file edit remains observable through inspect facts；an `outline` command with an explicit numeric `--limit` returns the complete three-entry fixture outline with `page: null`.

Supported by:
- `smoke|core:config-context|CORE-CONFIG-001`
