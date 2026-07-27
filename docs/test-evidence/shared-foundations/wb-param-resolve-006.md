### Case WB-PARAM-RESOLVE-006: Invalid append contributor blocks with observable provenance

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/invalid.rs > invalid_append_contributor_blocks_with_observable_provenance`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `invalid_append_contributor_blocks_with_observable_provenance` 直接验证“Invalid append contributor blocks with observable provenance”所描述的结果。
