### Case WB-PARAM-RESOLVE-002: Static default fills a missing source value

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/defaults.rs > static_default_fills_a_missing_source_value`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `static_default_fills_a_missing_source_value` 直接验证“Static default fills a missing source value”所描述的结果。
