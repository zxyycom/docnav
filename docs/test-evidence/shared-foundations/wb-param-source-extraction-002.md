### Case WB-PARAM-SOURCE-EXTRACTION-002: Env extractor reads declared values only and omits missing values

Entry:
- `crates/shared/cli-config-resolution/tests/canonical_core/env.rs > env_extractor_reads_declared_values_only_and_omits_missing_values`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Resolution core preserves normalized source facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `env_extractor_reads_declared_values_only_and_omits_missing_values` 直接验证“Env extractor reads declared values only and omits missing values”所描述的结果。
