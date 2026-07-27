### Case WB-NAV-OUTLINE-MODE-009: Path triggered default fallback reports non utf8 failure

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > path_triggered_default_fallback_reports_non_utf8_failure`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `path_triggered_default_fallback_reports_non_utf8_failure` 直接验证“Path triggered default fallback reports non utf8 failure”所描述的结果。
