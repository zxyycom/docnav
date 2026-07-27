### Case WB-PARAM-RESOLVE-016: Source rejects locator kind mismatch

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/source.rs > source_rejects_locator_kind_mismatch`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `source_rejects_locator_kind_mismatch` 直接验证“Source rejects locator kind mismatch”所描述的结果。
