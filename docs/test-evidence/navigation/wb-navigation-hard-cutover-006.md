### Case WB-NAVIGATION-HARD-CUTOVER-006: Hard cutover preserves field declaration order for primary diagnostic

Entry:
- `crates/shared/navigation/src/tests/navigation/hard_cutover.rs > hard_cutover_preserves_field_declaration_order_for_primary_diagnostic`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core catalog cutover preserves resolver parity”所涉及的稳定行为边界。

Proves:
- 原生入口 `hard_cutover_preserves_field_declaration_order_for_primary_diagnostic` 直接验证“Hard cutover preserves field declaration order for primary diagnostic”所描述的结果。
