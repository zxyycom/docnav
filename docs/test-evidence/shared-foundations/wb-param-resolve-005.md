### Case WB-PARAM-RESOLVE-005: Selected invalid candidate blocks materialization

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/invalid.rs > selected_invalid_candidate_blocks_materialization`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `selected_invalid_candidate_blocks_materialization` 直接验证“Selected invalid candidate blocks materialization”所描述的结果。
