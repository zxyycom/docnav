### Case WB-MD-META-004: Probe returns format evidence without navigation payload

Entry:
- `crates/adapters/markdown/tests/adapter/meta.rs > probe_returns_format_evidence_without_navigation_payload`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown manifest/probe/info 元数据稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `probe_returns_format_evidence_without_navigation_payload` 直接验证“Probe returns format evidence without navigation payload”所描述的结果。
