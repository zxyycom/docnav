### Case WB-CORE-ADAPTER-SURFACE-003: Dynamic adapter management is unsupported

Entry:
- `crates/docnav/src/cli/parser/tests/adapter_command.rs > dynamic_adapter_management_is_unsupported`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core adapter command surface 保持静态注册表边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `dynamic_adapter_management_is_unsupported` 直接验证“Dynamic adapter management is unsupported”所描述的结果。
