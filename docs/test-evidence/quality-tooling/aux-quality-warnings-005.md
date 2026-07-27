### Case AUX-QUALITY-WARNINGS-005: Warns when an accepted warning rule no longer matches any generated warning

Entry:
- `scripts/tools/quality-core/src/output/warnings/generator.test.ts > quality warning generation > warns when an accepted warning rule no longer matches any generated warning`

Contract:
- `docs/tooling.md` 定义或约束“Quality warning 阈值语义稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `quality warning generation > warns when an accepted warning rule no longer matches any generated warning` 直接验证“Warns when an accepted warning rule no longer matches any generated warning”所描述的结果。
