### Case WB-PARAM-RESOLVE-010: Deny conflict reports all source locators

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > deny_conflict_reports_all_source_locators`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `deny_conflict_reports_all_source_locators` 直接验证“Deny conflict reports all source locators”所描述的结果。
