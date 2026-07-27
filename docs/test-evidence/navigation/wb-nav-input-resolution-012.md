### Case WB-NAV-INPUT-RESOLUTION-012: Navigation rejects unselected explicit candidate after adapter routing

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/validation.rs > navigation_rejects_unselected_explicit_candidate_after_adapter_routing`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `navigation_rejects_unselected_explicit_candidate_after_adapter_routing` 直接验证“Navigation rejects unselected explicit candidate after adapter routing”所描述的结果。
