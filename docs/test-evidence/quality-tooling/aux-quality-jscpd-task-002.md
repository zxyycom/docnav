### Case AUX-QUALITY-JSCPD-TASK-002: Plans one scan task per code area

Entry:
- `scripts/tools/quality-core/src/measurement/scanners/jscpd/area-scans.test.ts > jscpd tasks > plans one scan task per code area`

Contract:
- `docs/tooling.md` 定义或约束“Quality jscpd task planning 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `jscpd tasks > plans one scan task per code area` 直接验证“Plans one scan task per code area”所描述的结果。
