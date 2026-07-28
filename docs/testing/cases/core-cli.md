# core-cli

## Case BB-CORE-ADAPTER-MGMT-001: Core adapter inspection 命令覆盖

Owner: `docs/cli.md#内置-adapter-检查`

Entities:
- `smoke|core:tool-commands|CORE-ADAPTER-MGMT-001`

Proves:
- `doctor` 报告 static registry 和 adapter layer checks。
- `adapter list` 输出 core release static registry 内置 Markdown adapter metadata。

## Case BB-CORE-ARGS-001: Core 拒绝缺失的 operation 参数

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `smoke|core:cli-argument-failure|CORE-ARGS-001`

Proves:
- document command 缺少本 operation 拥有的必需参数时返回稳定 input failure。
- 该 smoke case 代表这一类外部 CLI 错误，不枚举所有 parser 组合。

## Case BB-CORE-AUTO-READ-001: Core unique-ref auto-read 默认值与关闭来源可观察

Owner: `docs/navigation-input-resolution.md#unique-ref-auto-read-composition`

Entities:
- `smoke|core:auto-read|CORE-AUTO-READ-001`

Proves:
- 真实 `find` CLI 在所有 auto-read 来源省略且当前返回结果只有一个 distinct ref 时，默认以 `unique-ref` 追加 nested read；`protocol-json` 与 `readable-view` 从同一结果保留 ref、content type 和 nested content。
- CLI、默认 project config fixture 或显式 user config fixture 解析为 `disabled` 时，真实进程以退出码 `0` 返回原 base find，stdout 保持所选输出模式且 stderr 为空；代表性 protocol/readable 分支均不出现 `auto_read`，readable base projection 也不产生 block。

## Case BB-CORE-CONFIG-001: Config inspect source status 与参数事实可观察

Owner: `docs/cli.md#配置命令`

Entities:
- `smoke|core:config-context|CORE-CONFIG-001`
- `smoke|core:config-context|CORE-CONFIG-005`

Proves:
- `docnav config inspect` reports selected project/user source scope、origin、load state、source diagnostics and current adapter/output/pagination parameter facts without modifying either selected file。
- Inspect output includes the config-source projection entry for the observed pagination field.
- When config disables pagination, an explicit numeric limit does not re-enable page slicing and the complete representative outline remains observable.

## Case BB-CORE-CONFIG-002: Invalid config value 通过 inspect/source validation 被拒绝

Owner: `docs/cli.md#配置命令`

Entities:
- `smoke|core:config-context|CORE-CONFIG-002`

Proves:
- A selected config source containing `defaults.output: "text"` appears in `docnav config inspect` source diagnostics as field `defaults.output` with reason `enum_invalid`.

## Case BB-CORE-CONFIG-003: Legacy defaults.limit 通过 config source diagnostic 被拒绝

Owner: `docs/navigation-input-resolution.md#配置文件形状`

Entities:
- `smoke|core:config-context|CORE-CONFIG-003`

Proves:
- project config 中的 legacy `defaults.limit` 会在真实 `outline` CLI 链路中返回 config-owned `INVALID_REQUEST`。
- structured `unknown_config_field` / `config_issues` diagnostic 报告字段、source level、path origin 和 config path。

## Case BB-CORE-CONFIG-004: Adapter-scoped config 按 catalog operation applicability 生效

Owner: `docs/navigation-input-resolution.md#selected-operation-catalog-view`

Entities:
- `smoke|core:config-context|CORE-CONFIG-004`
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::core_linked_markdown_consumes_project_native_max_heading_level`

Proves:
- Project config 中的 `options.docnav-markdown.max_heading_level` 通过 core-authored Markdown-scoped catalog entry 影响 `outline` entries。
- User config 中的 `options.docnav-markdown.max_heading_level` 通过 direct config file edit/read 参与 source priority；当 catalog 不把该参数绑定到 selected operation 时，返回 structured unsupported diagnostic 并保留 source level/path。

## Case BB-CORE-CONFIG-PATH-001: Config path flags select CLI config targets

Owner: `docs/cli.md#配置文件路径`

Entities:
- `smoke|core:config-context|CORE-CONFIG-PATH-001`
- `smoke|core:config-context|CORE-CONFIG-PATH-002`

Proves:
- 真实 document operation 通过 `--project-config <path>` 和 `--user-config <path>` 使用显式 selected config files，而不是 project context、`DOCNAV_CONFIG_DIR` 或平台默认路径。
- `docnav config inspect --project-config <path> --user-config <path>` reports exactly those selected source paths and their origins without writing either file.
- Document operations and `config inspect` share the same config source descriptor/path selection boundary, while document operation value resolution remains owned by navigation input resolution.
- A removed mutating config subcommand is rejected at the CLI boundary and leaves both explicitly selected files byte-for-byte unchanged.

## Case BB-CORE-FAIL-001: Candidate probe failure 投影为格式候选摘要

Owner: `docs/adapter-contract.md#probe-识别`

Entities:
- `smoke|core:registry-contract-failures|CORE-FAIL-001`

Proves:
- candidate discovery 阶段的 built-in adapter probe failure 被报告为 `FORMAT_UNKNOWN` candidate summary。
- candidate failure 不会被折叠成 selected adapter layer failure。
- 未显式声明 adapter 的 automatic discovery 全部 probe 失败时，candidate failures 从属于 primary diagnostic details。

## Case BB-CORE-INFO-001: Core exposes Markdown info through readable output

Owner: `docs/output.md#readable-view`

Entities:
- `smoke|core:real-markdown-link-chain|CORE-INFO-001`

Proves:
- A real `docnav info` process succeeds through the linked Markdown adapter with empty stderr and no protocol envelope in readable output.
- The readable header derives a display containing the selected Markdown format and `text/markdown` content type.

## Case BB-CORE-LINK-001: Core 原样传递真实 Markdown ref

Owner: `docs/ref-contract.md#共享调用流程`

Entities:
- `smoke|core:real-markdown-link-chain|CORE-LINK-001`
- `smoke|core:real-markdown-link-chain|CORE-LINK-002`

Proves:
- 真实 `docnav` 进程可以通过 Markdown adapter 分别完成 `outline -> ref -> read` 和 `find -> ref -> read` 链路。
- outline/find 返回的 adapter ref 可原样提交给 read，`readable-view` read 保留该 ref；用户可见阅读文本不包含 protocol envelope。

## Case BB-CORE-MD-DOCHEAD-001: Markdown document head 通过真实 CLI 输出模式可观察

Owner: `docs/adapters/markdown.md#document-head`

Entities:
- `smoke|core:real-markdown-link-chain|CORE-MD-DOCHEAD-001`

Proves:
- 真实 CLI fixture 包含 YAML frontmatter、普通前导正文和可见 heading 时，structured outline 在 heading entries 前暴露 `HEAD:leading`。
- `protocol-json` 验证 raw document head entry facts：非空 `label`、`kind = document_head`、`location.line_start`、`metadata.document_region = leading` 和缺少 readable-only `display`。
- `readable-view` 验证 display、成本摘要和 read content block 由内置 renderer 从同一 `ProtocolResponse` 的 raw facts 与 read result 派生。
- 通过 `HEAD:leading` 执行 read 返回 document head 原文，`content_type` 为 `text/markdown`，并保留 frontmatter delimiter 与普通前导正文。

## Case BB-CORE-MD-OPTIONS-001: Markdown max_heading_level option 通过真实 CLI 生效

Owner: `docs/adapters/markdown.md#可见性与-max_heading_level`

Entities:
- `smoke|core:real-markdown-link-chain|CORE-MD-OPTIONS-001`
- `smoke|core:real-markdown-link-chain|CORE-MD-OPTIONS-002`
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::core_linked_markdown_delegates_native_option_range_to_adapter`

Proves:
- Markdown `max_heading_level` 可以从 CLI flag 影响 `outline` 可见粒度；越界值按 Markdown 声明的范围投影为带 explicit source 的 `range_invalid` option issue。Project config source 的同类型证明由 `BB-CORE-CONFIG-004` 承担。

## Case BB-CORE-OUTPUT-001: Core 文档输出模式不混层

Owner: `docs/output.md#输出层边界`

Entities:
- `smoke|core:document-output-boundary|CORE-OUTPUT-001`
- `smoke|core:document-output-boundary|CORE-OUTPUT-002`
- `smoke|core:document-output-boundary|CORE-OUTPUT-003`
- `smoke|core:document-output-boundary|CORE-OUTPUT-004`

Proves:
- 省略 output 和显式 `readable-view` 产生相同的 built-in readable-view text contract；`protocol-json` 对同一文档结果输出完整 protocol envelope。
- Readable text 与 protocol JSON 保持隔离：presentation-only display/cost/framing 不进入 protocol raw result，success/failure 仍分别保留当前 owner 的可观察语义。
- CLI 显式选择或 project config 选择 `protocol-json` 时，navigation response 产生前的 document failure 仍输出完整 failure envelope，不回退到 `readable-view`；project-selected 分支同时覆盖低优先级 user config 加载失败。
- CLI 显式使用已删除的 `readable-json` 时走普通 invalid-value boundary，不产生 alias、fallback 或 document output。
- Unstructured outline selected by config 在 `readable-view` 中作为 content block、在 `protocol-json` 中作为 raw result 可观察，并且不虚构 entries/ref/page/continuation。

## Case BB-CORE-REF-001: Adapter ref 错误穿过 Core

Owner: `docs/ref-contract.md#共享-ref-错误`

Entities:
- `smoke|core:real-markdown-ref-error|CORE-REF-001`

Proves:
- 被选中 adapter 拒绝的 ref 会从 core 返回稳定 protocol failure。
- `protocol-json` 承载错误时，stderr 不输出 JSON payload。

## Case BB-CORE-SELECT-001: 显式 adapter 失败返回 selection diagnostic

Owner: `docs/adapter-contract.md#adapter-选择`

Entities:
- `smoke|core:adapter-selection|CORE-SELECT-001`
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::missing_adapter_routing_precedes_invalid_native_option`

Proves:
- 显式 CLI 或 project config 选择的 adapter 不存在时返回 adapter selection diagnostic，不隐藏为 registry fallback。
- 显式 adapter id 不存在时，即使同一请求携带 invalid-looking native option，也返回 adapter selection diagnostic，而不是 option validation error。

## Case BB-CORE-SOURCE-001: Core adapter source 来自 static registry

Owner: `docs/adapter-contract.md#内置-adapter-接口`

Entities:
- `smoke|core:registry-contract-failures|CORE-SOURCE-001`

Proves:
- core release 内置 adapter dispatch 使用 static registry 中的 linked adapter implementation。
- 默认 document operation 的 implementation source 与项目配置中的普通文件内容解耦。

## Case BB-CORE-TOOLS-001: Core 非 document 命令保持可用

Owner: `docs/cli.md#命令面`

Entities:
- `smoke|core:tool-commands|CORE-TOOLS-001`
- `smoke|core:tool-commands|CORE-TOOLS-002`
- `smoke|core:tool-commands|CORE-TOOLS-003`

Proves:
- `init` 通过真实 CLI 创建 project config。
- `version` 输出 crate version，document help 暴露 output/pagination CLI options。

## Case WB-CORE-ADAPTER-001: Core 校验 adapter contract 对齐

Owner: `docs/adapter-contract.md#内置-adapter-接口`

Entities:
- `cargo|docnav:lib:docnav|registry::tests::adapter_layer_check_reports_definition_metadata_and_core_source`
- `cargo|docnav:lib:docnav|registry::tests::adapter_list_preserves_static_registry_projection`
- `cargo|docnav:lib:docnav|registry::tests::static_registry_contains_built_in_markdown_adapter`

Proves:
- Core static registry 包含 release 内置 Markdown adapter descriptor metadata。
- Adapter layer check 从 adapter definition 读取 metadata，并把 core-owned implementation source 保留为 static registry fact。
- `adapter list` preserves the same static-registry id and implementation-source projection.

## Case WB-CORE-ADAPTER-SURFACE-001: Core adapter command surface 保持静态注册表边界

Owner: `docs/cli.md#命令面`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::adapter_command::adapter_list_returns_static_registry_command`
- `cargo|docnav:lib:docnav|cli::parser::tests::adapter_command::dynamic_adapter_management_is_unsupported`

Proves:
- `adapter list` 解析为 static registry inspection command。
- 默认 adapter command surface 只接受 `adapter list` 作为 inspection command。

## Case WB-CORE-ARGS-001: Core parser 保持 operation 参数所有权

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::auto_read_rejects_missing_duplicate_and_inapplicable_input_structurally`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::duplicate_generated_single_value_flag_is_rejected_structurally`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::extra_document_positional_is_rejected`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::generated_value_flag_without_value_maps_clap_structural_error`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::max_heading_level_is_rejected_for_unsupported_operations`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::unknown_document_argument_is_rejected`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::structural_errors::unused_known_argument_value_is_rejected_before_execution`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::auto_read_modes_keep_the_canonical_identity_and_exact_tokens`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::explicit_max_heading_level_value_is_parsed_for_supported_operations`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::explicit_pagination_value_is_parsed`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::generated_page_keeps_canonical_identity_for_selected_validation`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::invalid_auto_read_token_is_preserved_for_selected_validation`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::values::invalid_pagination_token_is_preserved_for_selected_validation`

Proves:
- generated operation-owned 参数保留 canonical identity、CLI locator 与 normalized typed/invalid candidate，selected validation 继续负责 field-local 约束。
- generated valueless Boolean switch 与 value flags 都从 current static/generated Clap shape 派生 lexical facts，并由 Clap 捕获 normalized candidate。
- Clap 拥有 unknown、missing value 和 duplicate single-value structural failures；lexical compatibility boundary 只使用同源 cardinality facts 保持 positional 与 operation-inapplicable diagnostics。
- 未被当前 operation 使用的 known argument 不会被抢先 typed 解析，而是在 parser 边界返回 input diagnostic。

## Case WB-CORE-ARGS-REPAIR-001: Core parser input diagnostics expose protocol repair context

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::protocol_errors::extra_document_positional_protocol_error_has_repair_context`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::protocol_errors::unknown_document_argument_protocol_error_has_repair_context`
- `cargo|docnav:lib:docnav|cli::parser::tests::document_arguments::protocol_errors::unsupported_info_page_protocol_error_has_repair_context`

Proves:
- Unknown document flags、extra document positionals and operation-inapplicable known arguments produce parser diagnostics whose protocol-json error projection preserves reason、received token、expected context and repair guidance.

## Case WB-CORE-CONFIG-PATH-001: Core parser accepts config file path flags

Owner: `docs/cli.md#配置文件路径`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::config_inspect_parses_selected_config_file_paths`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::config_inspect_rejects_document_context_flags`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::config_path_flag_before_known_flag_is_missing_value_input_error`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::document_command_parses_config_file_paths_as_exact_values`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::init_and_doctor_parse_config_file_paths`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::init_rejects_user_config_path_flag`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::inline_config_path_value_can_start_with_known_flag_text`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::legacy_config_subcommands_are_rejected`
- `cargo|docnav:lib:docnav|cli::parser::tests::config_paths::unsupported_config_path_flag_is_input_error`

Proves:
- Core parser accepts `--project-config <path>` and `--user-config <path>` on document operations, `config`, and `doctor`; accepts `--project-config <path>` on `init`; and rejects missing values or use on undocumented commands before reading config sources or dispatching operations.
- Parsed document/config/doctor commands preserve both selected path flag values, while init preserves the supported project path value only.

## Case WB-CORE-CONFIG-PATH-002: Core config inspect uses selected config file paths

Owner: `docs/cli.md#配置命令`

Entities:
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_omits_optional_non_json_null_parameter_fact`
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_reports_catalog_adapter_range_with_exact_source`
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_reports_explicit_source_load_status_without_failing`
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_reports_selected_sources_and_parameter_facts_without_writing`
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_reports_validation_diagnostics_without_failing`
- `cargo|docnav:lib:docnav|config::commands::tests::config_inspect_serializes_complete_invalid_json_load_status`
- `cargo|docnav:lib:docnav|config::commands::tests::init_creates_and_preserves_selected_project_config_file`
- `cargo|docnav:lib:docnav|config::commands::tests::init_rejects_selected_project_config_directory`

Proves:
- Complete serialized-output goldens cover one valid selected project/user pair and one invalid-JSON project source. They lock source status、summaries、registry-backed config-source projection、resolved parameter facts、source-attributed diagnostics and top-level output shape while normalizing only runtime paths.
- The valid golden proves adapter-id native option projection and project/user/built-in provenance without modifying either selected file. The invalid-load golden proves invalid JSON remains a successful inspection result with matching source and parameter diagnostics.
- Explicit missing、top-level non-object and not-file source states retain focused equivalence checks；unreadable source loading remains owned by lower-layer config loading / parameter-resolution tests. Optional non-JSON config `null` suppresses its static default without creating a parameter fact.
- `init --project-config` creates or preserves the selected project config file and rejects an existing directory at that selected file path.

## Case WB-CORE-CONFIG-SOURCE-001: Core config source validation preserves navigation-owned fields

Owner: `docs/navigation-input-resolution.md#配置文件形状`

Entities:
- `cargo|docnav:lib:docnav|config::model::tests::defaults_auto_read_preserves_raw_modes`
- `cargo|docnav:lib:docnav|config::model::tests::native_options_config_accepts_generic_raw_map`
- `cargo|docnav:lib:docnav|config::store::tests::adapter_id_native_option_config_key_is_typed_validated`
- `cargo|docnav:lib:docnav|config::store::tests::bare_native_option_config_path_is_unknown`
- `cargo|docnav:lib:docnav|config::store::tests::default_missing_config_path_is_absent`
- `cargo|docnav:lib:docnav|config::store::tests::direct_config_file_rejects_empty_invocation_log_content_capture_root`
- `cargo|docnav:lib:docnav|config::store::tests::direct_config_file_rejects_empty_invocation_log_path`
- `cargo|docnav:lib:docnav|config::store::tests::explicit_missing_config_path_reports_blocking_issue`
- `cargo|docnav:lib:docnav|config::store::tests::invalid_adapter_id_native_option_value_is_rejected`
- `cargo|docnav:lib:docnav|config::store::tests::navigation_owned_outline_config_is_accepted`
- `cargo|docnav:lib:docnav|config::store::tests::nested_non_object_config_field_reports_structured_config_issue`
- `cargo|docnav:lib:docnav|config::store::tests::unknown_config_field_reports_structured_config_issue`

Proves:
- Core config source loading accepts documented navigation-owned `outline.mode_rules[]` and `outline.auto_full_read.thresholds[]` fields instead of rejecting them as unknown top-level config.
- Core config source validation preserves raw `outline` config for inspection/read purposes while validating core-authored defaults and adapter-scoped catalog config keys.
- Bare `options.max_heading_level` is rejected as an ordinary `unknown_config_field`; it is not migrated or interpreted as an adapter-id native option source path.
- Invalid adapter-id native option values and nested non-object fields produce structured source-attributed config issues.
- Default missing config paths remain absent, while explicit missing paths report `missing_explicit_cli` with explicit path origin.
- The core config model preserves raw auto-read modes and generic adapter-native option maps for later navigation-owned resolution.

## Case WB-CORE-DOCTOR-001: Doctor 聚合 typed check 退出码

Owner: `docs/cli.md#内置-adapter-检查`

Entities:
- `cargo|docnav:lib:docnav|config::doctor::tests::adapter_layer_failure_dominates_multiple_doctor_failures`
- `cargo|docnav:lib:docnav|config::doctor::tests::doctor_reports_explicit_missing_config_as_failure`

Proves:
- selected config file 失败使用其 typed input error 退出码。
- adapter layer failure 使用 adapter/protocol 退出码，并在多个失败同时存在时按严重度决定 doctor 退出码。

## Case WB-CORE-HELP-001: Core parser help/version 不进入 document output mode

Owner: `docs/cli.md#parser-与-help`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::help::help_command_has_no_output_mode`
- `cargo|docnav:lib:docnav|cli::parser::tests::help::help_returns_typed_help_command`
- `cargo|docnav:lib:docnav|cli::parser::tests::help::help_text_scopes_catalog_parameters_to_supported_operations`
- `cargo|docnav:lib:docnav|cli::parser::tests::help::help_text_shows_only_public_output_modes`
- `cargo|docnav:lib:docnav|cli::parser::tests::help::non_document_surfaces_keep_their_own_command_shapes`
- `cargo|docnav:lib:docnav|cli::parser::tests::help::version_command_has_no_output_mode`

Proves:
- `--help` 和 operation help 返回 typed help command，并且 document output 只展示 `readable-view` 与 `protocol-json`。
- Operation help 按 core catalog binding 展示当前 operation 可用的参数；例如 outline 展示 `--max-heading-level`，read 不展示。
- root help、version、config、adapter、init 和 doctor 保持各自的 static surface，不进入 document output mode。

## Case WB-CORE-INVOCATION-LOG-001: Core runtime invocation log 保持审计边界

Owner: `docs/cli.md#invocation-logging`

Entities:
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::config::invocation_cli_content_root_without_cli_log_does_not_override_config_log`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::config::invocation_cli_log_records_config_load_failure_before_runtime_config`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::config::invocation_log_config_type_error_is_blocking_core_config_error`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::config::invocation_logging_config_enabled_uses_validated_core_config`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::config::invocation_logging_disabled_creates_no_log_side_effect`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_auto_read_content_capture_reuses_root_event_and_hash_shape`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_capture_failure_does_not_change_operation_result`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_content_capture_writes_hash_named_file_and_event`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_failed_auto_read_keeps_only_the_successful_root_event`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_find_auto_read_logs_root_metadata_without_capture_file`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_read_metadata_only_hashes_content_without_capture_file`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::content::invocation_unwritable_log_path_does_not_change_operation_result`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::failure::invocation_failure_logs_bounded_layer_code_and_summary`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::failure::invocation_linked_handler_structured_diagnostic_logs_adapter_dispatch_failure`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::output::invocation_logging_enabled_success_writes_jsonl_with_request_id`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::output::invocation_output_write_failure_logs_output_projection_without_completion`
- `cargo|docnav:lib:docnav|runtime::tests::invocation_logging::output::invocation_readable_view_stdout_stays_free_of_log_events`

Proves:
- Invocation logging 默认关闭，且配置关闭时不创建日志副作用。
- CLI/config 显式启用后，core document operation 写入 JSONL operation event，并保留 request id、adapter id、operation status、bounded failure layer/code/summary 和 stdout purity。
- Config load failure 可由显式 CLI log 在 runtime config 初始化前记录为 config-layer failure。
- Metadata-only read event 只记录 SHA-256 content hash、content type 和 size metadata；未单独开启 content capture 时不写正文文件。
- 单独开启 content capture 后正文文件只写入独立 root 下的日期/`sha256-<content_hash>.content` 相对路径，文件 bytes hash 与主日志 hash 一致。
- Successful outline/find auto-read 仍只记录根 operation event；追加的 read content 复用既有 metadata-only content reference，显式 capture 时复用同一个 hash/capture event shape，未显式 capture 时不写正文文件。
- Unique-ref 已触发但 nested read 返回 adapter diagnostic 时，public/base command 仍成功；日志只保留单个 root `operation_completed`，不记录 nested diagnostic 或正文，也不产生 read root event 或 content capture。
- 日志文件写入失败、output projection failure 和 content capture failure 不改变原 document operation 的成功/失败语义。

## Case WB-CORE-DOCUMENT-PATH-001: Core normalizes linked-adapter document paths

Owner: `docs/cli.md#项目根与路径`

Entities:
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::linked_adapter_uses_absolute_document_path_from_project_subdir`

Proves:
- A relative path from a project subdirectory is normalized to the intended absolute document before linked-adapter dispatch.
- Internal normalized path shape does not leak into the protocol result.

## Case WB-CORE-OUTPUT-001: Core 输出编排保持通道边界

Owner: `docs/output.md#输出层边界`

Entities:
- `cargo|docnav:lib:docnav|output::tests::app_error_normalizes_non_protocol_diagnostic_before_document_output`
- `cargo|docnav:lib:docnav|output::tests::built_in_render_failure_uses_existing_core_error_id`
- `cargo|docnav:lib:docnav|output::tests::document_protocol_json_writes_protocol_envelope_with_empty_stderr`
- `cargo|docnav:lib:docnav|output::tests::document_readable_view_uses_shared_output_facade`
- `cargo|docnav:lib:docnav|output::tests::document_unstructured_outline_readable_view_uses_shared_output_facade`
- `cargo|docnav:lib:docnav|output::tests::non_document_json_writes_value_directly`
- `cargo|docnav:lib:docnav|output::tests::plain_text_outcome_writes_text_directly`
- `cargo|docnav:lib:docnav|output::tests::readable_error_uses_document_facade_and_exit_policy_stays_local`
- `cargo|docnav:lib:docnav|output::tests::readable_view_renderer_fatal_uses_bounded_stderr_and_internal_exit`
- `cargo|docnav:lib:docnav|output::tests::rendered_writer_failure_stays_an_io_failure`

Proves:
- Core 把 document success 和提前发生的 document failure 统一表示为 `ProtocolResponse` 后再执行 output plan。
- 省略 output 或显式 `readable-view` 构造携带内置 renderer 的 `Rendered`；`protocol-json` 构造 `ProtocolJson`，non-document output 保持 owner-specific。
- 内置 renderer failure 沿用现有 output failure/exit mapping：stdout 为空、stderr 诊断有界，不切换 plan 或 renderer。
- Core document output composition 保持 stdout、stderr 和 exit code 职责，并覆盖真实 CLI smoke 中观察到的两个 public document output modes。

## Case WB-CORE-OUTPUTMODE-001: Core parser document output mode 解析稳定

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `cargo|docnav:lib:docnav|cli::parser::tests::output::parse_explicit_protocol_json`
- `cargo|docnav:lib:docnav|cli::parser::tests::output::parse_without_output_has_none`
- `cargo|docnav:lib:docnav|cli::parser::tests::output::removed_output_value_remains_a_canonical_candidate_for_navigation_validation`

Proves:
- 未显式传入 `--output` 时 parser 不抢先解析默认值，由 document request/config chain 决定。
- `readable-view` 与 `protocol-json` 可解析；已删除的 `readable-json` 与其它合法值集合之外的 output value 返回普通可诊断 invalid-value error。

## Case WB-CORE-PARAMETER-CATALOG-001: Core parameter catalog authors canonical product facts

Owner: `docs/navigation-input-resolution.md#core-parameter-catalog`

Entities:
- `cargo|docnav:lib:docnav|parameter_catalog::tests::catalog_fields_do_not_enable_an_env_source`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::catalog_fields_preserve_current_locator_type_default_merge_and_range_facts`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::core_catalog_contains_the_auto_read_orchestration_parameter`
- `cargo|docnav:lib:docnav|parameter_catalog::tests::operation_projection_filters_only_by_closed_bindings`

Proves:
- The core catalog owns current locators, value kinds, defaults, merge rules, ranges, and auto-read orchestration facts without enabling an undeclared environment source.
- Selected operation projection filters fields only through closed catalog bindings.

## Case WB-CORE-PREFLIGHT-001: Core preflight 检测 protocol-json intent

Owner: `docs/cli.md#document-operation-执行`

Entities:
- `cargo|docnav:lib:docnav|cli::preflight::tests::detects_equals_protocol_json_output`
- `cargo|docnav:lib:docnav|cli::preflight::tests::detects_space_separated_protocol_json_output`
- `cargo|docnav:lib:docnav|cli::preflight::tests::document_without_output_defaults_to_readable_view`
- `cargo|docnav:lib:docnav|cli::preflight::tests::legacy_config_failure_uses_protocol_json_framing`
- `cargo|docnav:lib:docnav|cli::preflight::tests::non_document_output_context_keeps_plain_command_semantics`
- `cargo|docnav:lib:docnav|cli::preflight::tests::non_document_protocol_json_hint_uses_core_output_flag`
- `cargo|docnav:lib:docnav|cli::preflight::tests::projected_output_locator_frames_document_structural_failure`

Proves:
- Core preflight 从 current document command 的 canonical projection 获取 output locator/cardinality，并在解析失败前识别空格分隔和等号形式的 protocol-json intent。
- Structural document failure 使用 projected output intent 选择 protocol failure framing。
- Root 与 non-document commands 不触发 document projection；preflight 只服务错误输出模式选择，不替代正式 parser。

## Case WB-CORE-PROJECT-CONTEXT-001: Project context resolves explicit and platform config paths

Owner: `docs/cli.md#配置文件路径`

Entities:
- `cargo|docnav:lib:docnav|project_context::tests::explicit_config_paths_are_resolved_relative_to_invocation_cwd`
- `cargo|docnav:lib:docnav|project_context::tests::explicit_user_config_path_does_not_require_platform_default`
- `cargo|docnav:lib:docnav|project_context::tests::user_config_path_prefers_docnav_config_dir_then_platform_default`
- `cargo|docnav:lib:docnav|project_context::tests::user_config_path_uses_dot_docnav_under_platform_user_root`

Proves:
- Explicit project and user config paths resolve relative to the invocation working directory and do not require a platform default.
- Implicit user config selection prefers `DOCNAV_CONFIG_DIR` and otherwise resolves `.docnav` below the platform user root.
