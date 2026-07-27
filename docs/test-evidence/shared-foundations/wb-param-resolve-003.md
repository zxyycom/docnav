### Case WB-PARAM-RESOLVE-003: Dynamic default remains an observable source fact

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/resolution/defaults.rs > dynamic_default_remains_an_observable_source_fact`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `dynamic_default_remains_an_observable_source_fact` 直接验证“Dynamic default remains an observable source fact”所描述的结果。
