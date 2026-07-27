### Case BB-CORE-CONFIG-PATH-001: Config path flags select CLI config targets

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-CONFIG-PATH-001`

Contract:
- `docs/cli.md` 定义或约束“Config path flags select CLI config targets”所涉及的稳定行为边界。

Proves:
- 真实 document operation 通过 `--project-config <path>` 和 `--user-config <path>` 使用显式 selected config files，而不是 project context、`DOCNAV_CONFIG_DIR` 或平台默认路径。
- `docnav config inspect --project-config <path> --user-config <path>` reports exactly those selected source paths and their origins without writing either file.
- Document operations and `config inspect` share the same config source descriptor/path selection boundary, while document operation value resolution remains owned by navigation input resolution.
- Representative mutating legacy command `config set` with selected config path flags is rejected through the normal CLI parse/error boundary and does not modify selected files; the removed command names form one parser equivalence class.
