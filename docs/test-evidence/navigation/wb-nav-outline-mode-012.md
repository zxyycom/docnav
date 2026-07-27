### Case WB-NAV-OUTLINE-MODE-012: Unregistered outline rule key is rejected before rule parsing

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > unregistered_outline_rule_key_is_rejected_before_rule_parsing`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `unregistered_outline_rule_key_is_rejected_before_rule_parsing` 直接验证“Unregistered outline rule key is rejected before rule parsing”所描述的结果。
