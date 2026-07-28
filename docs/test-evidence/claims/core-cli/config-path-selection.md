# Claim CLAIM-CLI-CONFIG-PATH-SELECTION-001: Config path flags select CLI config targets

Topic: `core-cli`
Owner ref: `docs/cli.md#配置文件路径`

Statement:
- Explicit project and user config path flags select the exact files instead of context or platform defaults.

Observations:
- 真实 document operation 通过 `--project-config <path>` 和 `--user-config <path>` 使用显式 selected config files，而不是 project context、`DOCNAV_CONFIG_DIR` 或平台默认路径。
- `docnav config inspect --project-config <path> --user-config <path>` reports exactly those selected source paths and their origins without writing either file.
- Document operations and `config inspect` share the same config source descriptor/path selection boundary, while document operation value resolution remains owned by navigation input resolution.

Supported by:
- `smoke|core:config-context|CORE-CONFIG-PATH-001`
