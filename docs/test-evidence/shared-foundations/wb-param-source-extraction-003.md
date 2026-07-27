### Case WB-PARAM-SOURCE-EXTRACTION-003: Selected invalid env value preserves diagnostic facts

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/env.rs > selected_invalid_env_value_preserves_diagnostic_facts`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Resolution core preserves normalized source facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `selected_invalid_env_value_preserves_diagnostic_facts` 直接验证“Selected invalid env value preserves diagnostic facts”所描述的结果。
