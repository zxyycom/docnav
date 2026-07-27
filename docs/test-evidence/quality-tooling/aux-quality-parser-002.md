### Case AUX-QUALITY-PARSER-002: Parses scc 3.7 Provider paths and rejects unknown CSV headers

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality scanner output parsing > parses scc 3.7 Provider paths and rejects unknown CSV headers`

Contract:
- `docs/tooling.md` 定义或约束“Quality scanner parser fixtures 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality scanner output parsing > parses scc 3.7 Provider paths and rejects unknown CSV headers` 直接验证“Parses scc 3.7 Provider paths and rejects unknown CSV headers”所描述的结果。
