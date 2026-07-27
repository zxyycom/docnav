### Case WB-CONTRACTS-ERROR-001: Adapter contracts error mapping 保持 protocol 投影边界

Entry:
- `crates/shared/adapter-contracts/src/tests/error.rs > adapter_error_constructors_project_protocol_error_details`

Contract:
- `docs/adapter-contract.md` 定义或约束“Adapter contracts error mapping 保持 protocol 投影边界”所涉及的稳定行为边界。

Proves:
- Adapter document errors project to protocol error code, owner, location and default guidance through `AdapterError::protocol_error()`.
- Adapter-owned native option errors project issue metadata to invalid-request received, expected, details and guidance fields.
