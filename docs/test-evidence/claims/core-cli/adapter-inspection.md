# Claim CLAIM-CLI-ADAPTER-INSPECTION-001: Core adapter inspection 命令覆盖

Topic: `core-cli`
Owner ref: `docs/cli.md#内置-adapter-检查`

Statement:
- Built-in adapter inspection reports the static registry and adapter-layer health checks.

Observations:
- `doctor` 报告 static registry 和 adapter layer checks。
- `adapter list` 输出 core release static registry 内置 Markdown adapter metadata。

Supported by:
- `smoke|core:tool-commands|CORE-ADAPTER-MGMT-001`
