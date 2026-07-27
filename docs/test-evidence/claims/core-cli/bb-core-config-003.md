# Claim CLAIM-BB-CORE-CONFIG-003: Legacy defaults.limit 通过 config source diagnostic 被拒绝

Topic: `core-cli`
Owner ref: `docs/navigation-input-resolution.md#配置文件形状`

Statement:
- The removed defaults.limit field is rejected as invalid configuration rather than accepted as a pagination input.

Observations:
- project config 中的 legacy `defaults.limit` 会在真实 `outline` CLI 链路中返回 config-owned `INVALID_REQUEST`。
- structured `unknown_config_field` / `config_issues` diagnostic 报告字段、source level、path origin 和 config path。

Supported by:
- `smoke|core:config-context|CORE-CONFIG-003`
