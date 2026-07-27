### Case WB-NAV-OUTLINE-MODE-002: Project path rule overrides user rule and uses default utf8 fallback

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > project_path_rule_overrides_user_rule_and_uses_default_utf8_fallback`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `project_path_rule_overrides_user_rule_and_uses_default_utf8_fallback` 直接验证“Project path rule overrides user rule and uses default utf8 fallback”所描述的结果。
