# Claim CLAIM-BB-CORE-REF-001: Adapter ref 错误穿过 Core

Topic: `core-cli`
Owner ref: `docs/ref-contract.md#共享-ref-错误`

Statement:
- A ref rejected by the selected adapter crosses core as the stable shared ref failure.

Observations:
- 被选中 adapter 拒绝的 ref 会从 core 返回稳定 protocol failure。
- `protocol-json` 承载错误时，stderr 不输出 JSON payload。

Supported by:
- `smoke|core:real-markdown-ref-error|CORE-REF-001`
