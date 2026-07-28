# Claim CLAIM-ADAPTER-FULL-READ-DEFINITION-001: Adapter definition validation 收敛 full-read capability facts

Topic: `adapter-contracts`
Owner ref: `docs/adapter-contract.md#内置-adapter-接口`

Statement:
- Adapter definitions reject a declared unstructured full-read capability whose required capability facts are empty.

Observations:
- Adapter definition validation rejects a declared but empty unstructured full-read capability set.
- Adapter definition validation rejects blank or duplicate cost measurement units.

Supported by:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::definition::adapter_definition_rejects_empty_full_read_capabilities`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::definition::adapter_definition_rejects_invalid_full_read_cost_units`
