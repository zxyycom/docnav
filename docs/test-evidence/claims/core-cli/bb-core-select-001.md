# Claim CLAIM-BB-CORE-SELECT-001: 显式 adapter 失败返回 selection diagnostic

Topic: `core-cli`
Owner ref: `docs/adapter-contract.md#adapter-选择`

Statement:
- An explicitly selected missing adapter returns an adapter-selection diagnostic and does not fall back silently.

Observations:
- 显式 CLI 或 project config 选择的 adapter 不存在时返回 adapter selection diagnostic，不隐藏为 registry fallback。
- 显式 adapter id 不存在时，即使同一请求携带 invalid-looking native option，也返回 adapter selection diagnostic，而不是 option validation error。

Supported by:
- `smoke|core:adapter-selection|CORE-SELECT-001`
