### Case WB-NAV-OUTLINE-MODE-010: Invalid path rule returns source scoped diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > invalid_path_rule_returns_source_scoped_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `invalid_path_rule_returns_source_scoped_diagnostic` 直接验证“Invalid path rule returns source scoped diagnostic”所描述的结果。
