### Case WB-CONTRACTS-UNSTRUCTURED-004: Adapter definition dispatches closed standard input variants

Entry:
- `crates/shared/adapter-contracts/src/tests/operation_input.rs > adapter_definition_dispatches_closed_standard_input_variants`

Contract:
- `docs/adapter-contract.md` 定义或约束“Adapter contracts unstructured full-read hook defaults 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `adapter_definition_dispatches_closed_standard_input_variants` 直接验证“Adapter definition dispatches closed standard input variants”所描述的结果。
