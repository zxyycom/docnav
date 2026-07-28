# navigation

## Case WB-NAV-ADAPTER-SOURCE-001: Navigation adapter selection 保持静态来源边界

Owner: `docs/adapter-contract.md#adapter-选择`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::adapter_source::automatic_discovery_all_fail_projects_candidate_failures`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::adapter_source::explicit_missing_adapter_error_carries_invocation_failure_layer`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::adapter_source::explicit_missing_adapter_reports_static_registry_guidance`

Proves:
- 显式声明的 adapter id 不存在于 static registry 时返回 `ADAPTER_UNAVAILABLE`。
- diagnostic owner 来自 `docnav-navigation` routing，而不是 core routing。
- guidance 指向 current core release static registry。
- Automatic discovery 全部候选失败时返回 `FORMAT_UNKNOWN`，并把 routing-owned probe failure reason 投影到 primary details 的 `candidate_failures`。
- 本 case 不证明 discovery 顺序、extension metadata 排序或 manifest metadata 与 candidate failure 的关系。

## Case WB-NAV-AUTO-READ-001: Unique-ref auto-read eligibility and composition remain bounded

Owner: `docs/navigation-input-resolution.md#unique-ref-auto-read-composition`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|auto_read::tests::invalid_composed_response_falls_back_to_the_original_base`
- `cargo|docnav-navigation:lib:docnav_navigation|auto_read::tests::invalid_nested_success_is_not_accepted_as_a_read_result`
- `cargo|docnav-navigation:lib:docnav_navigation|auto_read::tests::unique_ref_ignores_empty_refs_and_uses_string_exact_deduplication`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::find::find_eligibility_keeps_empty_or_multiple_ref_base_results`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::find::repeated_find_refs_dispatch_one_read_on_later_pages_with_continuation`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::adapter_base_result_with_auto_read_is_rejected_before_composition`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::disabled_mode_does_not_dispatch_nested_read`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::nested_read_diagnostic_silently_keeps_the_validated_base_result`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::nested_read_reuses_the_effective_limit_when_pagination_is_disabled`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::outline_eligibility_keeps_non_unique_or_unstructured_base_results`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_composition::outline::unique_outline_ref_composes_read_with_the_selected_document_context`

Proves:
- Only one non-empty distinct ref is eligible for nested read; empty, multiple, unstructured, or disabled results keep the validated base response.
- Nested read reuses the selected document context and effective pagination facts, while nested diagnostics or invalid nested success never invalidate the original base success.
- An adapter-supplied base `auto_read` is rejected before navigation composes its own result.

## Case WB-NAV-AUTO-READ-CONFIG-001: Auto-read mode follows canonical source precedence

Owner: `docs/navigation-input-resolution.md#unique-ref-auto-read-composition`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::tests::auto_read_replace_trace_keeps_selected_overridden_and_builtin_provenance`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_config::auto_read_mode_resolves_with_cli_project_user_and_builtin_precedence`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_config::invalid_auto_read_cli_value_reports_the_canonical_flag_and_tokens`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_config::invalid_auto_read_config_is_attributed_to_its_source`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::auto_read_config::read_and_info_recognize_valid_config_without_projecting_auto_read`

Proves:
- CLI, project, user, and built-in auto-read candidates resolve through the canonical precedence chain with source provenance.
- Invalid CLI or config values retain their canonical locator and source attribution; read and info accept valid config without projecting an inapplicable auto-read result.

## Case WB-NAV-CATALOG-ASSOCIATIONS-001: Parameter catalog associations are total and scoped

Owner: `docs/navigation-input-resolution.md#core-parameter-catalog`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::associations::catalog_retains_known_adapter_ids_for_config_validation`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::associations::entry_adapter_must_be_known`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::associations::field_and_entry_associations_are_total_and_unique`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::associations::field_set_identity_and_locator_errors_are_preserved`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::associations::standalone_config_validation_consumes_catalog_scalar_fields`

Proves:
- Every catalog entry references one canonical field, uses a known adapter scope, and preserves field-set validation errors.
- Field and entry associations are total and unique, and standalone config validation consumes the same catalog scalar facts.

## Case WB-NAV-CATALOG-BINDINGS-001: Parameter catalog bindings are unambiguous and type-compatible

Owner: `docs/navigation-input-resolution.md#core-parameter-catalog`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::bindings::binding_value_kind_must_match_the_field_definition`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::bindings::every_entry_requires_one_unambiguous_binding_per_operation`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::bindings::navigation_and_core_only_bindings_enforce_their_target_value_kinds`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::bindings::standard_input_targets_are_unique_for_overlapping_adapter_scopes`

Proves:
- Each entry has one unambiguous binding per operation and each binding value kind matches its canonical field definition.
- Navigation-only, core-only, and standard-input targets preserve their closed value kinds; overlapping adapter scopes cannot claim the same standard input target.

## Case WB-NAV-INPUT-RESOLUTION-001: Navigation input resolution 保持来源解析边界

Owner: `docs/navigation-input-resolution.md#resolution-流程`

Entities:
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::core_linked_markdown_reports_project_and_user_native_option_sources`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::adapter_scopes::navigation_accepts_config_option_applicable_to_operation`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::adapter_scopes::navigation_does_not_forward_other_known_adapter_namespace`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::adapter_scopes::navigation_keeps_same_option_key_distinct_by_adapter_namespace`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::defaults::navigation_accepts_max_heading_level_range_boundaries`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::defaults::navigation_includes_adapter_native_option_default`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::defaults::optional_non_json_config_null_suppresses_default_projections`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::resolution::navigation_resolves_selected_catalog_option_and_dispatches`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::resolution::pagination_disabled_normalizes_protocol_and_standard_input_limit`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::resolution::read_and_find_build_sibling_protocol_and_standard_input_facts`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::resolution::resolved_protocol_options_and_standard_input_share_resolution_value`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_blocks_dispatch_when_native_option_type_cannot_materialize`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_blocks_invalid_catalog_value_for_other_known_adapter`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_rejects_config_option_not_applicable_to_operation`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_rejects_unknown_config_option_after_adapter_routing`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_rejects_unselected_explicit_candidate_after_adapter_routing`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_reports_explicit_native_option_type_failure`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_reports_explicit_range_failure_with_adapter_compatible_diagnostic`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_reports_typed_native_option_failure_with_source`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::validation::navigation_reports_unknown_adapter_id_under_options_as_config_diagnostic`

Proves:
- Navigation input resolution preserves source labels for explicit input and project config option issues.
- Navigation 接收带 canonical identity、locator 和 source facts 的 normalized CLI candidates；selected catalog members 在 request construction 和 dispatch 前进入 canonical resolution。
- The core-authored catalog controls adapter scope、operation applicability、typed validation and static defaults；navigation resolves selected fields without reconstructing those facts.
- Config source projection uses `options.<adapter-id>.<option-key>`; equal option keys in different adapter id namespaces stay distinct, and bare `options.<option-key>` is a normal unknown/invalid config path.
- Navigation consumes adapter-scoped values only from the selected adapter namespace for the selected operation; other known adapter namespaces are not forwarded to the selected strategy.
- Static catalog defaults affect the resolved operation result when no explicit/project value is provided.
- Protocol `OperationArguments` and closed `StandardOperationInput` are sibling projections of the same resolved values；optional config `null` suppresses both default projections.
- Unknown adapter namespaces、unknown selected options、operation-inapplicable options and invalid typed values remain blocking source-attributed diagnostics.

## Case WB-NAV-OUTLINE-MODE-001: Navigation outline_mode selectors and pre-dispatch stable

Owner: `docs/navigation-input-resolution.md#outline-mode-resolution`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::cost_threshold_triggers_hook_full_read_and_preserves_selector_cost`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::invalid_path_rule_returns_source_scoped_diagnostic`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::later_structured_path_rule_opts_out_of_cost_threshold`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::path_triggered_default_fallback_reports_non_utf8_failure`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::path_triggered_hook_result_facts_are_used`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::project_path_rule_overrides_user_rule_and_uses_default_utf8_fallback`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::threshold_adapter_mismatch_does_not_request_cost_measurements`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::threshold_filtering_and_unit_merge_keep_structured_when_minimum_not_met`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::threshold_missing_measurement_and_runtime_unavailable_fall_back_to_structured`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::unregistered_outline_config_key_returns_source_scoped_diagnostic`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::outline_mode::unregistered_outline_rule_key_is_rejected_before_rule_parsing`

Proves:
- path rules use deterministic source/order priority, can select unstructured full read, and can opt out to structured before cost thresholds run.
- adapter-scoped cost thresholds filter by selected adapter, merge same-unit thresholds to the minimum value, request only declared units, and fall back to structured when measurement is missing or unavailable.
- outline config source shape rejects an unregistered `outline.*` key, an unregistered `outline.mode_rules[]` item key, missing required members and invalid typed values before selector parsing and reports the source-scoped nested field path.
- Current owner-specific outline validation preserves parity across config inspect source validation, direct config read and navigation resolution; typed-fields compound helper tests are only added if this parity cannot be proven.
- unstructured full-read pre-dispatch skips the normal outline handler and returns either default UTF-8 content, adapter hook content with selector cost, or adapter hook result facts with stable `path_rule` / `cost_threshold` reasons.
- path-triggered default full-read returns a controlled non-UTF-8 failure instead of producing lossy content.

## Case WB-NAV-PROTOCOL-BRIDGE-001: Protocol requests map to closed navigation inputs

Owner: `docs/navigation-input-resolution.md#request-construction`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::protocol::protocol_dispatch_rejects_request_and_standard_input_operation_mismatch`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::protocol::protocol_request_maps_core_inputs_to_operation_arguments`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::protocol::protocol_request_maps_read_and_find_operation_shapes`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::protocol::protocol_request_rejects_missing_read_ref`

Proves:
- Protocol request fields map to the matching closed operation arguments and standard inputs for outline, read, find, and info.
- 缺失 read ref 或 request/input operation 不匹配时，在 dispatch 前失败。

## Case WB-NAV-RESPONSE-VALIDATION-001: Navigation response validation 保留 failure-layer 归因

Owner: `docs/architecture.md#调用链`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::protocol::response_validation_failure_carries_result_validation_layer`

Proves:
- Navigation 校验 base response 时，结果构造失败保留 `ResultValidation` failure-layer，并同时保留 selected adapter 和 request correlation facts。

## Case WB-NAVIGATION-CONFIG-SOURCES-002: Navigation loads config sources with descriptor origin

Owner: `docs/navigation-input-resolution.md#配置文件形状`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::default_missing_config_sources_are_absent_without_diagnostics`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::explicit_config_field_diagnostics_preserve_selected_source_path`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::explicit_config_path_selection_preserves_parameter_priority`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::explicit_config_value_diagnostics_preserve_selected_source_path`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::explicit_invalid_json_config_source_is_blocking_diagnostic`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::explicit_missing_config_source_is_blocking_diagnostic`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::navigation_loads_project_and_user_config_sources_from_descriptors`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::navigation_rejects_nested_non_object_config_shapes`

Proves:
- `docnav-navigation` loads project/user config sources from core-supplied descriptors that carry source level, resolved path and path origin.
- Default-path missing project/user config sources are absent without diagnostics.
- Explicit-path missing、unreadable、invalid JSON 和 top-level non-object config sources return blocking config source diagnostics with source level and selected config file path.
- Selecting a config file through CLI flag does not promote values inside that file to direct argv source; parameter priority remains `explicit > project > user > built_in`.

## Case WB-NAVIGATION-FIELD-SETS-001: Selected field set follows closed catalog applicability

Owner: `docs/navigation-input-resolution.md#selected-operation-catalog-view`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::projection::operation_applicability_is_derived_from_closed_bindings`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::catalog::tests::projection::selected_operation_projection_includes_common_and_exact_adapter_fields_only`
- `cargo|docnav-navigation:lib:docnav_navigation|parameters::fields::tests::selected_fields_combine_fixed_inputs_with_catalog_projection`

Proves:
- The selected operation field set combines fixed operation inputs with the core-authored parameter catalog projection.
- Adapter-scoped catalog fields are included only for the selected adapter；fields scoped to another adapter are excluded.
- Operation applicability 只从 closed bindings 派生。

## Case WB-NAVIGATION-HARD-CUTOVER-001: Core catalog cutover preserves resolver parity

Owner: `docs/navigation-input-resolution.md#resolution-流程`

Entities:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::hard_cutover::hard_cutover_preserves_common_and_native_option_source_priority`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::hard_cutover::removed_readable_json_cli_value_is_rejected_by_canonical_resolution`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::hard_cutover::valid_explicit_values_do_not_hide_invalid_lower_priority_config`

Proves:
- Normalized explicit `Source` carries core-catalog common and adapter-scoped candidates into navigation；explicit values retain priority over project and user values through the canonical resolver, and the public output mode/result remains unchanged.
- A valid higher-priority explicit common or adapter-scoped value does not hide an invalid project/user config candidate；the blocking diagnostic retains source level、selected config path and reason.
