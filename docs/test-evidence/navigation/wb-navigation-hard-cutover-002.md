### Case WB-NAVIGATION-HARD-CUTOVER-002: Hard cutover preserves common and native option source priority

Entry:
- `crates/shared/navigation/src/tests/navigation/hard_cutover.rs > hard_cutover_preserves_common_and_native_option_source_priority`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core catalog cutover preserves resolver parity”所涉及的稳定行为边界。

Proves:
- 原生入口 `hard_cutover_preserves_common_and_native_option_source_priority` 直接验证“Hard cutover preserves common and native option source priority”所描述的结果。
