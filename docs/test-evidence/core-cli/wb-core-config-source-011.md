### Case WB-CORE-CONFIG-SOURCE-011: Explicit missing config path reports blocking issue

Entry:
- `crates/docnav/src/config/store/tests.rs > explicit_missing_config_path_reports_blocking_issue`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core config source validation preserves navigation-owned fields”所涉及的稳定行为边界。

Proves:
- 原生入口 `explicit_missing_config_path_reports_blocking_issue` 直接验证“Explicit missing config path reports blocking issue”所描述的结果。
