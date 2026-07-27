### Case WB-PARAM-SERDE-002: Extracts only declared nested config path with source facts

Entry:
- `crates/shared/cli-config-resolution-serde/src/tests.rs > extracts_only_declared_nested_config_path_with_source_facts`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“serde config-path mapping preserves candidate facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `extracts_only_declared_nested_config_path_with_source_facts` 直接验证“Extracts only declared nested config path with source facts”所描述的结果。
