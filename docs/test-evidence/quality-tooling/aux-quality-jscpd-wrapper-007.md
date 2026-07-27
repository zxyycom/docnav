### Case AUX-QUALITY-JSCPD-WRAPPER-007: Classifies non zero jscpd exits as execution failures, not skipped scans

Entry:
- `scripts/tools/quality-core/src/measurement/scanners.test.ts > quality jscpd wrapper failure projection > classifies non-zero jscpd exits as execution failures, not skipped scans`

Contract:
- `docs/tooling.md` 定义或约束“Quality jscpd wrapper failure projection 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality jscpd wrapper failure projection > classifies non-zero jscpd exits as execution failures, not skipped scans` 直接验证“Classifies non zero jscpd exits as execution failures, not skipped scans”所描述的结果。
