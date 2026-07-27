### Case AUX-QUALITY-JSCPD-WRAPPER-006: Keeps real duplicate findings non fatal and normalizes jscpd JSON

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality jscpd wrapper failure projection > keeps real duplicate findings non-fatal and normalizes jscpd JSON`

Contract:
- `docs/tooling.md` 定义或约束“Quality jscpd wrapper failure projection 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality jscpd wrapper failure projection > keeps real duplicate findings non-fatal and normalizes jscpd JSON` 直接验证“Keeps real duplicate findings non fatal and normalizes jscpd JSON”所描述的结果。
