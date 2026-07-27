### Case WB-PARAM-RESOLVE-017: Resolver rejects an unknown field candidate

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/source.rs > resolver_rejects_an_unknown_field_candidate`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical resolution preserves one ordered merge chain”所涉及的稳定行为边界。

Proves:
- 原生入口 `resolver_rejects_an_unknown_field_candidate` 直接验证“Resolver rejects an unknown field candidate”所描述的结果。
