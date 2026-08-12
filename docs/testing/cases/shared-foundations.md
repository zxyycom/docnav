# shared-foundations

## Case WB-CLIARGS-BOUNDARY-001: Strict CLI 参数扫描保持输入边界

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::switch_flags_are_retained_without_consuming_value`
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::unknown_flag_does_not_consume_following_positional`
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::unused_value_flag_records_fact_without_validating_value`
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::unused_value_flag_requires_a_value`
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::used_value_flag_is_retained_and_consumes_value`
- `cargo|docnav-cli-args:lib:docnav_cli_args|tests::used_value_flag_requires_a_value_before_known_value_flag`

Proves:
- unknown flag 不消费后续 positional，used value flag 保留值，unused value flag 记录 operation applicability failure。
- switch flag、missing value、extra positional 和 unknown token 边界保持可观察，并可映射为 input diagnostic。

## Case WB-JSONIO-WRITE-001: JSON writer 保持格式和错误分类

Owner: `docs/protocol.md#编码`

Entities:
- `cargo|docnav-json-io:lib:docnav_json_io|tests::compact_json_writes_value_and_newline`
- `cargo|docnav-json-io:lib:docnav_json_io|tests::pretty_json_writes_value_and_newline`
- `cargo|docnav-json-io:lib:docnav_json_io|tests::serialization_failures_are_distinct_from_write_failures`
- `cargo|docnav-json-io:lib:docnav_json_io|tests::write_failures_are_reported`

Proves:
- compact/pretty JSON 都以换行结束。
- serialization failure 和 writer failure 保持不同错误类型。

## Case WB-PARAM-FIELD-CONTRACT-001: Canonical FieldDefSet preserves parameter declaration invariants

Owner: `crates/shared/typed-fields/README.md#docnav-typed-fields`

Entities:
- `cargo|cli-config-resolution:test:canonical_core|facade::primary_facade_builds_constrained_canonical_parameters`
- `cargo|docnav-typed-fields:test:canonical_parameters|canonical_processing_metadata_exposes_source_locators`
- `cargo|docnav-typed-fields:test:canonical_parameters|config_only_field_builds_without_cli_metadata`
- `cargo|docnav-typed-fields:test:canonical_parameters|field_build_rejects_invalid_cli_metadata_declarations`
- `cargo|docnav-typed-fields:test:canonical_parameters|field_lookup_uses_canonical_final_value_validation`
- `cargo|docnav-typed-fields:test:canonical_parameters|merge_strategy_is_canonical_field_metadata`
- `cargo|docnav-typed-fields:test:canonical_parameters|set_build_rejects_duplicate_and_invalid_source_locators`

Proves:
- One `FieldDefSet` exposes declared CLI flag、environment variable and config path locators from canonical processing metadata；optional CLI help、value name and Boolean encoding survive builder clone、declaration type erasure and aggregation beside canonical field facts.
- Definition-set build rejects duplicate processing locators、empty locator values、invalid dotted identities、invalid/duplicate CLI metadata attachments and incompatible、incomplete or ambiguous Boolean encodings with public errors；config-only fields remain valid without CLI metadata.
- `MergeStrategy` is canonical `FieldDef` metadata, defaults to `Replace`, and rejects strategies incompatible with the declared value kind.
- Canonical field lookup performs final value validation and returns a typed value or a stable wrong-type failure.

## Case WB-PARAM-RESOLVE-001: Canonical resolution preserves one ordered merge chain

Owner: `docs/navigation-input-resolution.md#resolution-流程`

Entities:
- `cargo|cli-config-resolution:test:canonical_core|facade::canonical_parameter_set_drives_env_resolution`
- `cargo|cli-config-resolution:test:canonical_core|resolution::defaults::dynamic_default_remains_an_observable_source_fact`
- `cargo|cli-config-resolution:test:canonical_core|resolution::defaults::static_default_fills_a_missing_source_value`
- `cargo|cli-config-resolution:test:canonical_core|resolution::invalid::invalid_append_contributor_blocks_with_observable_provenance`
- `cargo|cli-config-resolution:test:canonical_core|resolution::invalid::overridden_invalid_candidate_is_trace_only`
- `cargo|cli-config-resolution:test:canonical_core|resolution::invalid::selected_invalid_candidate_blocks_materialization`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::append_applies_canonical_constraints_only_after_merging_contributors`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::append_merge_preserves_source_order_and_provenance`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::deny_conflict_accepts_equal_values`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::deny_conflict_reports_all_source_locators`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::map_merge_preserves_source_order_and_provenance`
- `cargo|cli-config-resolution:test:canonical_core|resolution::merge::merged_value_is_revalidated`
- `cargo|cli-config-resolution:test:canonical_core|resolution::missing::missing_required_value_returns_no_partial_values`
- `cargo|cli-config-resolution:test:canonical_core|resolution::precedence::higher_priority_source_wins`
- `cargo|cli-config-resolution:test:canonical_core|resolution::precedence::later_source_wins_at_equal_priority`
- `cargo|cli-config-resolution:test:canonical_core|source::resolver_rejects_an_unknown_field_candidate`
- `cargo|cli-config-resolution:test:canonical_core|source::source_rejects_locator_kind_mismatch`

Proves:
- Higher priority wins；at equal priority, the later registered source wins deterministically. Static defaults automatically fall back, while an explicit dynamic-default source remains an ordinary source fact.
- `Replace`、`Append`、`MapMerge` and `DenyConflict` apply in deterministic low-to-high source order；append/map contributors and all deny-conflict locators remain observable in provenance/diagnostics.
- Canonical constraints are applied to the final merged value. Selected or merge-contributing invalid candidates block materialization, while an overridden invalid replacement remains trace-only.
- Missing required values and final validation failures return diagnostics and prevent partial `FieldValueMap` materialization.

## Case WB-PARAM-SERDE-001: serde config-path mapping preserves candidate facts

Owner: `docs/navigation-input-resolution.md#配置文件形状`

Entities:
- `cargo|cli-config-resolution-serde:lib:cli_config_resolution_serde|tests::extracts_only_declared_nested_config_path_with_source_facts`
- `cargo|cli-config-resolution-serde:lib:cli_config_resolution_serde|tests::missing_path_or_non_object_intermediate_produces_no_candidate`
- `cargo|cli-config-resolution-serde:lib:cli_config_resolution_serde|tests::non_config_locator_returns_a_public_error`
- `cargo|cli-config-resolution-serde:lib:cli_config_resolution_serde|tests::present_null_false_and_empty_containers_produce_candidates`

Proves:
- Only `ConfigPath` metadata declared by the canonical `FieldDefSet` is queried；extra config entries and missing paths produce no candidate.
- Present `null`、`false`、empty array and empty object values each remain present candidates with their JSON structure and config-path locator intact.
- A non-object intermediate behaves as an absent declared path；using a non-config processing locator returns a public `ConfigExtractionError` instead of panicking.

## Case WB-PARAM-SOURCE-EXTRACTION-001: Resolution core preserves normalized source facts

Owner: `docs/navigation-input-resolution.md#输入来源`

Entities:
- `cargo|cli-config-resolution:test:canonical_core|env::env_extractor_reads_declared_values_only_and_omits_missing_values`
- `cargo|cli-config-resolution:test:canonical_core|env::selected_invalid_env_value_preserves_diagnostic_facts`

Proves:
- Environment extraction queries only declared `EnvVar` locators；unknown environment entries are ignored and missing declared variables produce no candidate.
- Decodable values become normalized candidates；decode failures retain raw input、reason、source id and environment locator and block when selected or merge-contributing.
- `Source` exposes source kind、priority and candidate locator facts. CLI and structured-config extraction are proven by their companion cases.

## Case WB-TEXT-COST-001: Shared text cost helper 保持纯文本边界

Owner: `docs/architecture.md#共享库`

Entities:
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::bounded_meter_matches_logically_joined_fragments`
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::bounded_meter_stops_only_after_proven_exceed`
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::byte_cost_counts_utf8_bytes`
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::line_cost_counts_empty_unicode_and_trailing_newline`
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::requested_text_cost_dispatches_one_unit`
- `cargo|docnav-text-cost:lib:docnav_text_cost|tests::token_cost_uses_o200k_base_ordinary_text`

Proves:
- shared text cost helper functions 只接收纯文本并返回 unscoped protocol-compatible `Measurement`。
- `line_cost`、`byte_cost` 和 `token_cost` 分别固定 `lines`、`bytes`、`tokens` unit，并覆盖空文本、Unicode bytes、换行和 plain-text `o200k_base` token counting。
- Requested-unit `TextMeter` 把有序 fragments 作为一个逻辑文本流计量，结果与连接后的完整文本 helper 一致，并且只有在能够证明 threshold 已超出时才提前停止。

## Case WB-OUTPUT-SESSION-001: Shared output session 组合逐项输出职责

Owner: `docs/architecture.md#共享库`

Entities:
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::canonical_loop_does_not_request_tail_after_stop`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::caller_owned_projection_composes_with_vec_collector`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::collector_finish_failure_ends_session_without_output`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::empty_limited_session_finishes_complete`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::exact_limit_is_incomplete_when_source_has_more_input`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::exact_limit_accepts_input_and_stops`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::gate_policy::input_cost_failure_does_not_commit_input`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::gate_policy::invalid_fitting_cost_does_not_commit_input_or_budget`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::gate_policy::limited_gate_supplies_selected_unit_and_remaining_threshold`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::limited_session_commits_only_accepted_inputs`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::projection_stops_after_meter_proves_exceed`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::stopped_session_returns_error_without_committing`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::unbounded_report_uses_caller_owned_source_completion`
- `cargo|docnav-output-session:lib:docnav_output_session|tests::composition::unbounded_reuses_session_and_custom_collector_without_input_cost`

Proves:
- Limited Gate 拥有构造时传入的 unit/limit，把所选 unit 与当前 remaining threshold 传给 InputCost，并以结构化 outcome 表达原子 accepted/rejected 与 continue/stop；只有 accepted input exactly once 到达 Collector，measurement error、rejected input 和 stopped-session push 不提交内容。
- Limited 与 Unbounded 复用同一个 Session/Collector 形状；Unbounded 不需要 InputCost，Collector 可以形成 String、Vec 或 operation-specific typed output。
- Caller-owned structured TextProjection 可以选择计量字段并与泛型 Vec Collector 组合；Projection 能在 Meter 证明 exceed 后停止提供后续 fragments。Canonical producer loop 收到 stop 后不再请求 tail；Limited exact exhaustion 和 Unbounded 都由 caller-owned source completion 决定最终 complete。
- Empty source 可以形成 complete empty output；Collector finish failure 在 Session 形成 output 前原样返回。

## Case WB-TYPED-FIELDS-001: Typed field definition core 保持字段级不变量

Owner: `crates/shared/typed-fields/README.md#docnav-typed-fields`

Entities:
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::field_model::builder_exposes_schema_metadata_and_validates_values`
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::field_model::json_validation_accepts_any_json_value_including_null`
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::field_model::required_and_enum_constraints_are_driven_by_field_declarations`
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::field_model::validation_failures_keep_field_attribution`

Proves:
- Builder 生成 field identity、processing strategy-backed structured path、`FieldValidation<T>`、typed default metadata 和 schema metadata view，并能把合法 JSON value 校验为 typed value。
- Field metadata validation 区分 missing optional、wrong type 和 range violation，并保留 field identity、field path 和 machine-readable reason。
- Required enum field declaration 使用 Rust enum metadata 校验 allowed value，missing required 和 disallowed enum value 返回可诊断 validation failure。

## Case WB-TYPED-FIELDS-PROCESSING-ID-COMPILE-001: ProcessingId has no unchecked string conversion

Owner: `crates/shared/typed-fields/README.md#docnav-typed-fields`

Entities:
- `cargo|docnav-typed-fields:test:processing_id_compile|processing_id_has_no_unchecked_from_conversion`

Proves:
- The public `ProcessingId` boundary does not provide an unchecked string `From` conversion that could bypass non-empty validation.

## Case WB-TYPED-FIELDS-PROCESSING-001: Typed field processing declaration validation

Owner: `crates/shared/typed-fields/README.md#docnav-typed-fields`

Entities:
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::processing::field_build_rejects_duplicate_processing_id`
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::processing::processing_id_try_from_rejects_empty_value`
- `cargo|docnav-typed-fields:lib:docnav_typed_fields|tests::processing::set_build_rejects_missing_processing_strategy`

Proves:
- `ProcessingId::try_from` rejects empty and whitespace-only input.
- `FieldDef` build rejects duplicate processing identities.
- `FieldDefSet` build rejects a leaf declaration without a processing strategy and preserves the declaration path in the build error.
