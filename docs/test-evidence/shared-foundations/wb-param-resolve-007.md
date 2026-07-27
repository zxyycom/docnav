### Case WB-PARAM-RESOLVE-007: Append merge preserves source order and provenance

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/merge.rs > append_merge_preserves_source_order_and_provenance`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `append_merge_preserves_source_order_and_provenance` 直接验证“Append merge preserves source order and provenance”所描述的结果。
