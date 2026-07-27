### Case WB-CONTRACTS-DEFINITION-001: Adapter definition validation 收敛 full-read capability facts

Entry:
- `crates/shared/adapter-contracts/src/tests/definition.rs > adapter_definition_rejects_invalid_full_read_capabilities`

Contract:
- `docs/adapter-contract.md` 定义或约束“Adapter definition validation 收敛 full-read capability facts”所涉及的稳定行为边界。

Proves:
- Adapter definition validation rejects a declared but empty unstructured full-read capability set.
- Adapter definition validation rejects blank or duplicate cost measurement units.
