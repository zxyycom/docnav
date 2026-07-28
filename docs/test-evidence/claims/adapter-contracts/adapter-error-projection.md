# Claim CLAIM-ADAPTER-ERROR-PROJECTION-001: Adapter contracts error mapping 保持 protocol 投影边界

Topic: `adapter-contracts`
Owner ref: `docs/adapter-contract.md#文档操作执行边界`

Statement:
- Adapter-layer document and native-option errors preserve their typed protocol facts when projected.

Observations:
- Adapter document errors project to protocol error code, owner, location and default guidance through `AdapterError::protocol_error()`.
- Adapter-owned native option errors project issue metadata to invalid-request received, expected, details and guidance fields.

Supported by:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_document_not_found_error_projects_protocol_details`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::error::adapter_native_option_error_projects_protocol_details`
