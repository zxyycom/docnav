# output-rendering

## Case WB-OUTPUT-DOCUMENT-001: 共享 document output facade 分层

Owner: `docs/output.md#输出层边界`

Entities:
- `cargo|docnav-output:lib:docnav_output|tests::custom_renderer_receives_failure_response`
- `cargo|docnav-output:lib:docnav_output|tests::custom_renderer_receives_success_response_and_controls_exact_text`
- `cargo|docnav-output:lib:docnav_output|tests::protocol_json_serializes_success_and_failure_responses_without_rendering`
- `cargo|docnav-output:lib:docnav_output|tests::render_failure_happens_before_stdout_and_strategy_runs_once`
- `cargo|docnav-output:lib:docnav_output|tests::writer_failure_after_rendering_stays_a_writer_error`

Proves:
- `ProtocolJson` 与 `Rendered(RenderStrategy)` 共同消费 success/failure `ProtocolResponse`；protocol path 不调用 renderer，rendered path 只调用 plan 携带的 renderer。
- Custom renderer 成功时 stdout 精确等于其返回的完整 UTF-8 text，不由 output facade 追加 framing 或换行。
- `RenderFailure` 发生在第一次 stdout write 前，保持 stdout 为空且不调用 fallback renderer；渲染成功后的 writer failure 保持独立 I/O failure。

## Case WB-READABLE-VIEW-001: Readable-view conformance vectors preserve public framing

Owner: `docs/output.md#readable-view`

Entities:
- `cargo|docnav-readable:test:conformance_tests|conformance_01_no_block_outline`
- `cargo|docnav-readable:test:conformance_tests|conformance_04_single_block`
- `cargo|docnav-readable:test:conformance_tests|conformance_07_chinese`
- `cargo|docnav-readable:test:conformance_tests|conformance_10_crlf_payload`
- `cargo|docnav-readable:test:conformance_tests|conformance_11_no_trailing_newline`
- `cargo|docnav-readable:test:conformance_tests|conformance_12_block_marker_in_body`
- `cargo|docnav-readable:test:conformance_tests|conformance_14_readable_error`
- `cargo|docnav-readable:test:conformance_tests|conformance_15_error_guidance_array`
- `cargo|docnav-readable:test:conformance_tests|conformance_16_undeclared_extension_fields`
- `cargo|docnav-readable:test:conformance_tests|conformance_17_order_independent_assertions`
- `cargo|docnav-readable:test:conformance_tests|conformance_18_renderer_failure_missing_pointer`
- `cargo|docnav-readable:test:conformance_tests|conformance_19_renderer_failure_non_string`
- `cargo|docnav-readable:test:conformance_tests|conformance_20_outline_unstructured_content_block`
- `cargo|docnav-readable:test:conformance_tests|conformance_21_outline_auto_read_nested_content_block`

Proves:
- Committed conformance vectors cover header-only, content-block, Unicode, CRLF, trailing-newline, embedded-marker, error, extension-field, and auto-read presentations.
- Conformance assertions compare semantic header values and exact block framing without treating JSON member order as a contract.
- Invalid block pointers and non-string block targets remain renderer failures rather than silently changing output shape.

## Case WB-READABLE-RENDERER-001: 内置 readable renderer private block/framing 规则

Owner: `docs/output.md#readable-view`

Entities:
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::combined_character_utf8_byte_length`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::crlf_payload_preserved_in_block`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::emoji_utf8_byte_length`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::empty_string_block_zero_bytes`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::find_operation_no_blocks`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::framing_uses_lf_byte`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::header_json_is_valid_standalone`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::info_operation_no_blocks`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::multiple_blocks_with_nested_pointer`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::no_trailing_lf_payload_gets_framing_lf`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::outline_no_blocks_emits_header_only`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::payload_contains_block_marker_text`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::read_content_block`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::readable_error_block`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::to_readable_value_serializes_valid_payload`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::trailing_lf_payload_no_extra_framing_lf`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::undeclared_fields_preserved_in_header`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::success::utf8_byte_length_is_correct`

Proves:
- 内置 renderer 的 private presentation helper 保持 readable-view header、block replacement、UTF-8 byte length、LF framing、extension fields 和 operation-specific block/no-block config。
- Conformance representatives 保持 successful auto-read 的 `/auto_read/read/content` nested block、无 `auto_read` 的 structured outline header-only projection，以及 unstructured outline 的 `/content` base block。
- Private readable error value 和 header standalone JSON 可还原为最终 readable-view text；该 helper value 不形成 public output mode 或 schema。

## Case WB-READABLE-RENDERER-002: 内置 readable renderer private config/error 边界稳定

Owner: `docs/output.md#readable-view`

Entities:
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::errors::duplicate_pointer_in_config_fails`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::errors::non_string_target_fails`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::errors::pointer_missing_from_value_fails`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::errors::pointer_without_leading_slash_fails_config_validation`
- `cargo|docnav-readable:lib:docnav_readable|renderer::tests::errors::render_error_uses_stable_id`

Proves:
- renderer 可以区分 missing pointer、non-string target、duplicate pointer 和 pointer syntax。
- renderer failure 使用稳定 error id `readable_view_render_failed`。

## Case WB-OUTPUT-READABLE-MAPPING-001: 内置 readable-view 从 ProtocolResponse 派生

Owner: `docs/output.md#readable-view`

Entities:
- `cargo|docnav-output:lib:docnav_output|tests::auto_read::built_in_renderer_maps_find_auto_read_response`
- `cargo|docnav-output:lib:docnav_output|tests::auto_read::protocol_json_and_readable_view_share_outline_auto_read_facts`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_failure_response_and_preserves_block_framing`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_find_response`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_info_response`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_read_response_and_preserves_block_framing`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_structured_outline_response`
- `cargo|docnav-output:lib:docnav_output|tests::built_in_renderer_maps_unstructured_outline_response_and_preserves_block_framing`

Proves:
- Typed success `ProtocolResponse` representatives 覆盖 structured outline、unstructured outline、read、find 和 info 到 built-in readable-view 的 mapping；failure response 覆盖 readable failure presentation。
- Structured outline/find 的 `display`、read 的成本摘要、info summary、unstructured raw facts 和 failure fields 都从 response 语义派生。
- Block assertions 按 JSON `Value` 比较 header 语义，并精确验证 LF separator、block start marker、UTF-8 byte length、payload、end marker 和唯一尾部 LF；header member order 不属于断言。
- Outline/find auto-read keeps one protocol fact set across protocol JSON and readable-view, with nested read content rendered through the configured block boundary.
