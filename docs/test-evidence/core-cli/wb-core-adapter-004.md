### Case WB-CORE-ADAPTER-004: Adapter list preserves static registry projection

Entry:
- `crates/docnav/src/registry/tests.rs > adapter_list_preserves_static_registry_projection`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core 校验 adapter contract 对齐”所涉及的稳定行为边界。

Proves:
- 原生入口 `adapter_list_preserves_static_registry_projection` 直接验证“Adapter list preserves static registry projection”所描述的结果。
