### Case WB-CORE-ADAPTER-002: Static registry contains built in markdown adapter

Entry:
- `crates/docnav/src/registry/tests.rs > static_registry_contains_built_in_markdown_adapter`

Contract:
- `docs/adapter-contract.md` 定义或约束“Core 校验 adapter contract 对齐”所涉及的稳定行为边界。

Proves:
- 原生入口 `static_registry_contains_built_in_markdown_adapter` 直接验证“Static registry contains built in markdown adapter”所描述的结果。
