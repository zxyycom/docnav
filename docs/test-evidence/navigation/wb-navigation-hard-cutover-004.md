### Case WB-NAVIGATION-HARD-CUTOVER-004: Removed readable json cli value is rejected by canonical resolution

Entry:
- `crates/shared/navigation/src/tests/navigation/hard_cutover.rs > removed_readable_json_cli_value_is_rejected_by_canonical_resolution`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Core catalog cutover preserves resolver parity”所涉及的稳定行为边界。

Proves:
- 原生入口 `removed_readable_json_cli_value_is_rejected_by_canonical_resolution` 直接验证“Removed readable json cli value is rejected by canonical resolution”所描述的结果。
