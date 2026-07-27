### Case WB-CORE-CONFIG-SOURCE-009: Nested non object config field reports structured config issue

Entry:
- `crates/docnav/src/config/store/tests.rs > nested_non_object_config_field_reports_structured_config_issue`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `nested_non_object_config_field_reports_structured_config_issue` 直接验证“Nested non object config field reports structured config issue”所描述的结果。
