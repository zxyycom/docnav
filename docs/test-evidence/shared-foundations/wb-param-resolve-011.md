### Case WB-PARAM-RESOLVE-011: Map merge preserves source order and provenance

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > map_merge_preserves_source_order_and_provenance`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `map_merge_preserves_source_order_and_provenance` 直接验证“Map merge preserves source order and provenance”所描述的结果。
