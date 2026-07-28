# adapter-contracts

## Case WB-CONTRACTS-DEFINITION-001: Adapter definition validation 收敛 full-read capability facts

Owner: `docs/adapter-contract.md#内置-adapter-接口`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::definition::adapter_definition_rejects_empty_full_read_capabilities`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::definition::adapter_definition_rejects_invalid_full_read_cost_units`

Proves:
- Adapter definition validation rejects a declared but empty unstructured full-read capability set.
- Adapter definition validation rejects blank or duplicate cost measurement units.

## Case WB-CONTRACTS-ERROR-001: Adapter contracts error mapping 保持 protocol 投影边界

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_document_not_found_error_projects_protocol_details`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_native_option_error_projects_protocol_details`

Proves:
- Adapter document errors project to protocol error code, owner, location and default guidance through `AdapterError::protocol_error()`.
- Adapter-owned native option errors project issue metadata to invalid-request received, expected, details and guidance fields.

## Case WB-CONTRACTS-STANDARD-INPUT-001: Adapter contracts expose closed standard operation inputs

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::adapter_definition_dispatches_closed_standard_input_variants`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::standard_input_bindings_report_operation_and_expected_value_kind`

Proves:
- Standard operation input uses closed operation-specific variants instead of an open option map.
- Adapter definitions dispatch the matching closed input variant, while each standard binding reports its operation and expected value kind.

## Case WB-CONTRACTS-UNSTRUCTURED-001: Adapter contracts unstructured full-read hook defaults 稳定

Owner: `docs/adapter-contract.md#内置-adapter-接口`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::full_read::unstructured_full_read_hooks_default_to_absent_capabilities`

Proves:
- Adapter contract default unstructured full-read capabilities are absent unless the adapter opts in.
- Default unstructured full-read content hook returns an adapter error, cost measurement returns an empty `Cost`, and result facts return defaults.
