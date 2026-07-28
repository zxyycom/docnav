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
