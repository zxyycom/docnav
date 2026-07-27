### Case WB-NAV-OUTLINE-MODE-008: Path triggered hook result facts are used

Entry:
- `crates/shared/navigation/src/tests/navigation/outline_mode.rs > path_triggered_hook_result_facts_are_used`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation outline_mode selectors and pre-dispatch stable”所涉及的稳定行为边界。

Proves:
- 原生入口 `path_triggered_hook_result_facts_are_used` 直接验证“Path triggered hook result facts are used”所描述的结果。
