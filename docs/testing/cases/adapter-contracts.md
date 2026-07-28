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

## Case WB-CONTRACTS-NATIVE-001: Adapter-scoped native option declarations remain canonical

Owner: `docs/navigation-input-resolution.md#core-parameter-catalog`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::projection::operation_applicability_is_derived_from_closed_bindings`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::projection::selected_operation_projection_includes_common_and_exact_adapter_fields_only`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::adapter_scopes::navigation_keeps_same_option_key_distinct_by_adapter_namespace`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::catalog_fields_preserve_current_locator_type_default_merge_and_range_facts`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::core_catalog_contains_the_auto_read_orchestration_parameter`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::operation_projection_filters_only_by_closed_bindings`

Proves:
- The core-authored catalog declares the Markdown native option as one canonical typed field with its CLI flag, adapter-id config path, integer range, static default and exact adapter tag.
- Closed bindings author outline/find applicability; selected projections include common plus exact-adapter fields and exclude wrong-adapter or wrong-operation declarations.
- Equal option keys in different adapter namespaces remain distinct instead of reconstructing or merging adapter semantics downstream.

## Case WB-CONTRACTS-STANDARD-INPUT-001: Adapter contracts expose closed standard operation inputs

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::adapter_definition_dispatches_closed_standard_input_variants`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::standard_input_bindings_report_operation_and_expected_value_kind`
- `cargo|docnav-adapter-contracts:lib:docnav_adapter_contracts|tests::operation_input::standard_operation_input_exposes_closed_operation_specific_values`

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
