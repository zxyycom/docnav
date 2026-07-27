# Claim CLAIM-CLI-PROBE-FAILURE-PROJECTION-001: Candidate probe failure 投影为格式候选摘要

Topic: `core-cli`
Owner ref: `docs/adapter-contract.md#probe-识别`

Statement:
- A built-in adapter probe failure remains visible in the format-candidate failure projection.

Observations:
- candidate discovery 阶段的 built-in adapter probe failure 被报告为 `FORMAT_UNKNOWN` candidate summary。
- candidate failure 不会被折叠成 selected adapter layer failure。
- 未显式声明 adapter 的 automatic discovery 全部 probe 失败时，candidate failures 从属于 primary diagnostic details。

Supported by:
- `smoke|core:registry-contract-failures|CORE-FAIL-001`
