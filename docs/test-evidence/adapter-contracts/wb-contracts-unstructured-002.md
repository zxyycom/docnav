### Case WB-CONTRACTS-UNSTRUCTURED-002: Unstructured full read hooks default to absent capabilities

Entry:
- `crates/shared/adapter-contracts/src/tests/full_read.rs > unstructured_full_read_hooks_default_to_absent_capabilities`

Contract:
- `docs/adapter-contract.md` 定义或约束“Adapter contracts unstructured full-read hook defaults 稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `unstructured_full_read_hooks_default_to_absent_capabilities` 直接验证“Unstructured full read hooks default to absent capabilities”所描述的结果。
