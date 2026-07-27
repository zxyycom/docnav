# Claim CLAIM-CLI-TOOL-COMMANDS-001: Core 非 document 命令保持可用

Topic: `core-cli`
Owner ref: `docs/cli.md#命令面`

Statement:
- Non-document tool commands remain available through the same top-level CLI surface.

Observations:
- `init` 通过真实 CLI 创建 project config。
- `version` 输出 crate version，document help 暴露 output/pagination CLI options。

Supported by:
- `smoke|core:tool-commands|CORE-TOOLS-001`
