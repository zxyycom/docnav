# protocol

## Case WB-PROTO-BASIC-001: Protocol 基础类型和 envelope 规则稳定

Owner: `docs/protocol.md#协议字段与生命周期`

Entities:
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::base_result_constructors_omit_auto_read`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::constructs_find_auto_read_success_with_base_fields_and_outer_operation`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::constructs_outline_auto_read_success_with_base_fields_and_outer_operation`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::constructs_outline_success_response`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::constructs_unstructured_outline_success_response`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::failure_response_rules_preserve_or_null_operation`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::generated_request_id_is_non_empty`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::positive_integer_constructors_reject_zero`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::options::options_preserve_the_plain_json_object_wire_shape`

Proves:
- positive integer、non-empty generated request id、success response 和 failure operation preservation 保持协议基础不变量。
- outline success response coverage includes structured and unstructured discriminator branches, including the unstructured no entries/ref/page/continuation boundary.
- Outline/find auto-read preserves the outer operation and base fields, base constructors omit absent auto-read, and protocol options retain their plain JSON-object wire shape.

## Case WB-PROTO-DECODE-001: Protocol decode wrapper 返回可达阶段结果

Owner: `docs/protocol.md#请求包装`

Entities:
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_manifest_returns_the_typed_current_manifest`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_probe_result_returns_semantic_error_with_typed_value`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_protocol_request_preserves_defaultable_arguments`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_protocol_request_rejects_unmapped_arguments`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_protocol_request_runs_contract_before_raw_decode`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::decode::decode_protocol_response_keeps_operation_result_pairing_semantic`

Proves:
- Protocol request decoding runs schema/field-contract validation before raw typed decode.
- Protocol request decoding rejects unmapped request arguments at the schema stage.
- Protocol request decoding preserves defaultable empty arguments for operation-specific later resolution.
- Manifest wrapper returns the current typed manifest shape.
- Probe result semantic validation and protocol response operation/result pairing remain semantic-stage failures.

## Case WB-PROTO-DIAGNOSTICS-001: Protocol diagnostic mapping and projection 保持稳定

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::navigation_routing_default_guidance_uses_static_registry_language`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::protocol_error_codes_use_diagnostic_categories`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::protocol_error_location_uses_config_issue_path_and_field`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::basic::protocol_error_roundtrips_through_diagnostic_record_projection`

Proves:
- request、document、adapter-boundary 和 internal category 各有一个 protocol diagnostic code 代表，其 diagnostic projection rule 暴露对应 protocol code。
- Navigation routing protocol errors expose static-registry guidance, and protocol errors round-trip through `DiagnosticRecord` projection while preserving guidance.
- Invalid-request records with config issue details project protocol owner, location and received value from the diagnostic record.

## Case WB-PROTO-SCHEMA-001: Protocol fixtures 和 schema constraints 被实现测试消费

Owner: `docs/protocol.md#schema-所有权`

Entities:
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::manifest_contract_rejects_schema_backed_field_failures`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::parses_protocol_fixtures_into_shared_types`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::probe_contract_rejects_schema_backed_field_failures`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::probe_schema_rejects_missing_reasons_and_bad_confidence`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_auto_read_contract_accepts_exact_outline_and_find_success_objects`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_auto_read_contract_rejects_status_error_and_extra_fields`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_auto_read_contract_rejects_unstructured_read_and_info_placement`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_request_contract_rejects_schema_backed_field_failures`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_request_schema_rejects_an_empty_required_string`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_response_contract_rejects_schema_backed_field_failures`
- `cargo|docnav-protocol:lib:docnav_protocol|tests::schema::protocol_response_public_schema_rejects_undocumented_format_candidates`

Proves:
- 作为两条 output paths 统一输入的 success/failure `ProtocolResponse` fixtures 通过既有 public JSON Schema、runtime typed contract validation，并 deserialize 为共享 protocol types。
- protocol request、protocol response、manifest 和 probe 的 unknown fields、missing required fields、wrong types、version constants、field constraints 和 semantic boundary 被实现测试消费。
