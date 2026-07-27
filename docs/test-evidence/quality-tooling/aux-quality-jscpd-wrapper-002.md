### Case AUX-QUALITY-JSCPD-WRAPPER-002: Does not treat a successful jscpd run without JSON as a successful empty scan

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality jscpd wrapper failure projection > does not treat a successful jscpd run without JSON as a successful empty scan`

Contract:
- `docs/tooling.md` 定义或约束“Quality jscpd wrapper failure projection 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality jscpd wrapper failure projection > does not treat a successful jscpd run without JSON as a successful empty scan` 直接验证“Does not treat a successful jscpd run without JSON as a successful empty scan”所描述的结果。
