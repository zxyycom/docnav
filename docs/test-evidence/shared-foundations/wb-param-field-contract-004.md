### Case WB-PARAM-FIELD-CONTRACT-004: Field build rejects invalid cli metadata declarations

Entry:
- `crates/shared/typed-fields/tests/canonical_parameters.rs > field_build_rejects_invalid_cli_metadata_declarations`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Canonical FieldDefSet preserves parameter declaration invariants”所涉及的稳定行为边界。

Proves:
- 原生入口 `field_build_rejects_invalid_cli_metadata_declarations` 直接验证“Field build rejects invalid cli metadata declarations”所描述的结果。
