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
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_document_content_invalid_error_projects_exact_protocol_details`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_document_not_found_error_projects_protocol_details`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_native_option_error_projects_protocol_details`

Proves:
- Adapter document errors project to protocol error code, owner, location and default guidance through `AdapterError::protocol_error()`.
- Adapter content errors expose only normalized path and one stable JSON content-failure reason through `AdapterError::protocol_error()`.
- Adapter-owned native option errors project issue metadata to invalid-request received, expected, details and guidance fields.

## Case WB-CONTRACTS-STANDARD-INPUT-001: Adapter contracts expose closed standard operation inputs

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::adapter_document_dispatches_closed_standard_input_variants`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::standard_input_bindings_report_operation_and_expected_value_kind`

Proves:
- Standard operation input uses closed operation-specific variants instead of an open option map.
- An invocation-private adapter document dispatches the matching closed input variant, while each standard binding reports its operation and expected value kind.

## Case WB-CONTRACTS-REF-CONFORMANCE-001: Opaque refs round-trip on same and fresh adapter documents

Owner: `docs/adapter-contract.md#适配器职责`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::ref_conformance::ref_conformance_reads_opaque_ref_on_same_and_fresh_documents_at_page_one`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::ref_conformance::ref_conformance_rejects_mismatched_read_results`

Proves:
- The shared conformance helper forwards an opaque ref unchanged to page-one read on the existing adapter document and a fresh document created by the same definition.
- Both reads must return complete public-contract-valid success results with exact ref echo；schema-invalid content type 等 typed-but-invalid response 与 mismatched ref 都会失败，helper 不解析 refs 或 standardize pagination 和 adapter-owned correspondence。
- Adapter documents may retain non-`Send`、non-`Sync` private state；the factory and closed read input expose no state handle or generic parser/model parameter.

## Case WB-CONTRACTS-UNSTRUCTURED-001: Adapter contracts unstructured full-read hook defaults 稳定

Owner: `docs/adapter-contract.md#内置-adapter-接口`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::full_read::unstructured_full_read_hooks_default_to_absent_capabilities`

Proves:
- Adapter contract default unstructured full-read capabilities are absent unless the adapter opts in.
- Default adapter-document unstructured full-read content hook returns an adapter error, cost measurement returns an empty `Cost`, and result facts return defaults.
