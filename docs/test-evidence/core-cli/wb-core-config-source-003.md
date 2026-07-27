### Case WB-CORE-CONFIG-SOURCE-003: Adapter id native option config key is typed validated

Entry:
- `crates/docnav/src/config/store/tests.rs > adapter_id_native_option_config_key_is_typed_validated`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `adapter_id_native_option_config_key_is_typed_validated` 直接验证“Adapter id native option config key is typed validated”所描述的结果。
