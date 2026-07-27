### Case BB-CORE-CONFIG-002: Invalid config value 通过 inspect/source validation 被拒绝

Entry:
- `test/smoke/core/cases/config-management.ts > smoke task CORE-CONFIG-002`

Contract:
- `docs/cli.md` 定义或约束“Invalid config value 通过 inspect/source validation 被拒绝”所涉及的稳定行为边界。

Proves:
- A selected config source containing `defaults.output: "text"` appears in `docnav config inspect` source diagnostics as field `defaults.output` with reason `enum_invalid`.
