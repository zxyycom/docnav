# diagnostics

## Case WB-DIAG-RECORD-001: Diagnostic record finalization validates summary and typed details

Owner: `docs/architecture.md#共享库`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::detail_rule_rejects_one_missing_and_extra_field`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::detail_rule_validates_each_supported_field_type_once`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::invalid_request_details_accept_known_optional_context_fields`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::record::diagnostic_record_finalization_enforces_summary_and_code_defaults`

Proves:
- Diagnostic details rules accept each supported field type and the known optional `INVALID_REQUEST` context fields, and reject a missing required field or an extra field.
- For a valid draft, `DiagnosticRecordDraft::into_record()` preserves code defaults, typed details, source and absent guidance in the finalized primary record.
- `DiagnosticRecordDraft::into_record()` rejects an empty summary.

## Case WB-DIAG-RULES-001: Diagnostics code rules 保持稳定

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|code::rules::tests::diagnostic_rule_tables_follow_enum_order`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::code_rules::diagnostic_code_rules_cover_each_variant`

Proves:
- 通过 ordinal 索引的 protocol/boundary rule table 与对应 diagnostic code enum 保持逐项对齐。
- `DiagnosticCode::all()` exposes the current diagnostic registry, including representative protocol and boundary diagnostic codes.
- Each registry code exposes a non-empty unique stable string、non-empty details rule 和可用的 diagnostic projection route。

## Case WB-DIAG-CONTENT-001: Document content diagnostics expose exact stable details

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::document_content_invalid_details_require_exact_path_and_reason`

Proves:
- `DOCUMENT_CONTENT_INVALID` is a stable protocol diagnostic whose canonical details require `path` and one of the four documented JSON content-failure reasons.
- The exact details rule rejects a missing reason and parser-specific extra detail；typed deserialization rejects reasons outside the stable enum instead of exposing parser material.

## Case WB-DIAG-ADAPTER-001: Adapter unavailable diagnostics expose exact lookup facts

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::adapter_unavailable_details_require_exact_lookup_facts`

Proves:
- `ADAPTER_UNAVAILABLE` typed details serialize the declared adapter id and resolved selection source with fixed `ADAPTER_NOT_FOUND` reason and `resolve` stage.
- Typed deserialization rejects another reason or stage and rejects a missing selection source instead of admitting arbitrary lookup diagnostics.
