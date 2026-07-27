### Case WB-PARAM-RESOLVE-004: Overridden invalid candidate is trace only

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/invalid.rs > overridden_invalid_candidate_is_trace_only`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `overridden_invalid_candidate_is_trace_only` 直接验证“Overridden invalid candidate is trace only”所描述的结果。
