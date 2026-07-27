### Case WB-NAV-ADAPTER-SOURCE-002: Explicit missing adapter reports static registry guidance

Entry:
- `crates/shared/navigation/src/tests/navigation/adapter_source.rs > explicit_missing_adapter_reports_static_registry_guidance`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation adapter selection 保持静态来源边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_missing_adapter_reports_static_registry_guidance` 直接验证“Explicit missing adapter reports static registry guidance”所描述的结果。
