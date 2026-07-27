### Case WB-PARAM-FIELD-CONTRACT-003: Config only field builds without cli metadata

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > config_only_field_builds_without_cli_metadata`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_only_field_builds_without_cli_metadata` 直接验证“Config only field builds without cli metadata”所描述的结果。
