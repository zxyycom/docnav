# Claim CLAIM-CLI-CONFIG-VALUE-DIAGNOSTIC-001: Invalid config value 通过 inspect/source validation 被拒绝

Topic: `core-cli`
Owner ref: `docs/cli.md#配置命令`

Statement:
- Config inspection attributes invalid selected-source values to their field and validation reason.

Observations:
- A selected config source containing `defaults.output: "text"` appears in `docnav config inspect` source diagnostics as field `defaults.output` with reason `enum_invalid`.

Supported by:
- `smoke|core:config-context|CORE-CONFIG-002`
