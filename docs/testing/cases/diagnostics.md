# diagnostics

## Case WB-DIAG-RECORD-001: Diagnostic record construction validates typed details

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::detail_rule_rejects_one_missing_and_extra_field`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::detail_rule_validates_each_supported_field_type_once`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::details::invalid_request_details_accept_known_optional_context_fields`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::record::diagnostic_record_rejects_empty_summary`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::record::diagnostic_record_validates_details_and_uses_code_defaults`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::record::format_record_accepts_candidate_failures`

Proves:
- `DiagnosticRecordDraft::into_record()` creates primary records with code defaults, typed details, source and absent guidance preserved.
- Record construction rejects empty summaries and erased details whose shape does not match the diagnostic code.
- Format diagnostic details can carry subordinate `candidate_failures` in the primary record details object.

## Case WB-DIAG-RULES-001: Diagnostics code rules 保持稳定

Owner: `docs/protocol.md#协议错误对象`

Entities:
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|code::rules::tests::diagnostic_rule_tables_follow_enum_order`
- `cargo|docnav-diagnostics:lib:docnav_diagnostics|tests::code_rules::diagnostic_code_rules_cover_each_variant`

Proves:
- `DiagnosticCode::all()` exposes the current diagnostic registry, including representative protocol and boundary diagnostic codes.
- Each registry code exposes a non-empty unique stable string、non-empty details rule 和可用的 diagnostic projection route。
