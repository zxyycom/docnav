### Case WB-CORE-CONFIG-SOURCE-005: Invalid adapter id native option value is rejected

Entry:
- `crates/docnav/src/config/store/tests.rs > invalid_adapter_id_native_option_value_is_rejected`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `invalid_adapter_id_native_option_value_is_rejected` 直接验证“Invalid adapter id native option value is rejected”所描述的结果。
