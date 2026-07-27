### Case AUX-QUALITY-PARSER-005: Classifies invalid jscpd JSON and duplicate items as parse failures

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality scanner output parsing > classifies invalid jscpd JSON and duplicate items as parse failures`

Contract:
- `docs/tooling.md` 定义或约束“Quality scanner parser fixtures 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality scanner output parsing > classifies invalid jscpd JSON and duplicate items as parse failures` 直接验证“Classifies invalid jscpd JSON and duplicate items as parse failures”所描述的结果。
