### Case WB-CONTRACTS-UNSTRUCTURED-003: Standard operation input exposes closed operation specific values

Entry:
- `crates/shared/adapter-contracts/src/tests/operation_input.rs > standard_operation_input_exposes_closed_operation_specific_values`

Contract:
- `docs/adapter-contract.md` 定义或约束“Adapter contracts unstructured full-read hook defaults 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `standard_operation_input_exposes_closed_operation_specific_values` 直接验证“Standard operation input exposes closed operation specific values”所描述的结果。
