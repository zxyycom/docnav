### Case AUX-QUALITY-REPORT-003: Sorts rankings by metric without mutating scanner output order

Entry:
- `scripts/tools/quality-core/src/output/report/markdown-report.test.ts > quality report > sorts rankings by metric without mutating scanner output order`

Contract:
- `docs/tooling.md` 定义或约束“Quality report 排名和 changed-file 摘要稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality report > sorts rankings by metric without mutating scanner output order` 直接验证“Sorts rankings by metric without mutating scanner output order”所描述的结果。
