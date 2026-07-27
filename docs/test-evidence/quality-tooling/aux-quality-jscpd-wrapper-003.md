### Case AUX-QUALITY-JSCPD-WRAPPER-003: Classifies empty jscpd JSON reports as report failures

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality jscpd wrapper failure projection > classifies empty jscpd JSON reports as report failures`

Contract:
- `docs/tooling.md` 定义或约束“Quality jscpd wrapper failure projection 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality jscpd wrapper failure projection > classifies empty jscpd JSON reports as report failures` 直接验证“Classifies empty jscpd JSON reports as report failures”所描述的结果。
