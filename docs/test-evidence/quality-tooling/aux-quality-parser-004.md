### Case AUX-QUALITY-PARSER-004: Parses jscpd version and JSON output

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality scanner output parsing > parses jscpd version and JSON output`

Contract:
- `docs/tooling.md` 定义或约束“Quality scanner parser fixtures 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality scanner output parsing > parses jscpd version and JSON output` 直接验证“Parses jscpd version and JSON output”所描述的结果。
