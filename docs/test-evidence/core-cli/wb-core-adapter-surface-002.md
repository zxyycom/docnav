### Case WB-CORE-ADAPTER-SURFACE-002: Adapter list returns static registry command

Entry:
- `crates/docnav/src/cli/parser/tests/adapter_command.rs > adapter_list_returns_static_registry_command`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core adapter command surface 保持静态注册表边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `adapter_list_returns_static_registry_command` 直接验证“Adapter list returns static registry command”所描述的结果。
