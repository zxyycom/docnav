# json-adapter

## Case WB-JSON-MANIFEST-001: JSON 私有 manifest 声明固定格式身份

Owner: `docs/adapters/json.md#交付与公共边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::manifest_declares_fixed_json_identity`

Proves:
- adapter-private manifest 通过语义校验，并固定声明 adapter id `docnav-json`、format id `json`、suffixes `.json` / `.code-workspace`、exact filenames `.prettierrc` / `.watchmanconfig` 和 content type `application/json`。

## Case WB-JSON-SELECTED-PARSE-001: Selected JSON strategy parses the actual document

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::selected_outline_parses_actual_document_independently_of_path_hint`

Proves:
- Once the JSON definition is selected, definition dispatch parses the actual BOM-prefixed strict JSON document and returns JSON-owned outline facts even when the pathname itself supplies no JSON hint.

## Case WB-JSON-OUTLINE-001: JSON Adapter outline strategy 投影 common entries 与有限分页

Owner: `docs/adapters/json.md#outline`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_projects_mixed_json_to_exact_common_entries`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_handles_empty_container_roots_and_root_scalar`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_tiny_pages_preserve_complete_refs_and_terminate`

Proves:
- trait-dispatched `outline` operation 把 mixed JSON 按确定性 preorder 投影为只含 common fields 的 structured entries，保留完整 canonical ref、decoded object label、array index label 和六种 value kind；空 key 的正常 label 是两个双引号字符 `""` 且 ref 是 `json:#/`，Unicode key 保留 decoded label。
- empty container root 返回空 terminal entries，root scalar 返回唯一 root entry；tiny limit 下完整 ref 与 kind 保留，预算截断后没有可见正常 label 内容时只把 label 压缩为最小非空 fallback `.`，page 单调前进，结果耗尽及超过末页时 terminal。

## Case WB-JSON-READ-001: JSON Adapter read strategy 保留 selected spelling、ref、cost 与 Unicode pagination

Owner: `docs/adapters/json.md#read`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_round_trips_outline_refs_and_formats_selected_values`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_paginates_unicode_and_keeps_complete_cost_on_every_page`

Proves:
- trait-dispatched `read` operation 接受 outline 返回的 refs，并保留输入 ref 与 `application/json` content type；selected root、object、empty container、string、boolean 和 root scalar 使用确定性 structured spelling，number 保留原始 source token。
- Unicode-safe pagination 不切断 scalar，page 单调前进并可重组完整 selected content；每页及超过末页请求均保留分页前完整 cost，结果耗尽时 terminal。

## Case WB-JSON-DIAGNOSTICS-001: Selected JSON content failures use stable document diagnostics

Owner: `docs/adapters/json.md#错误边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::selected_outline_maps_current_document_failures_to_stable_diagnostics`
- `smoke|core:real-json|CORE-JSON-FAIL-001`

Proves:
- A selected JSON outline observes the document view it actually opens; a missing file and invalid UTF-8 preserve the existing exact document diagnostics.
- Invalid syntax、trailing non-whitespace input、duplicate decoded member and depth overflow each return `DOCUMENT_CONTENT_INVALID` with only the normalized path and the corresponding stable JSON reason.
- Real CLI explicit `--adapter docnav-json` selection on non-JSON `.md` paths returns all four selected JSON `DOCUMENT_CONTENT_INVALID` reasons above and does not fall back to pathname routing or another adapter.

## Case WB-JSON-PARSE-001: JSON loader 限定完整 UTF-8 单值输入

Owner: `docs/adapters/json.md#pathname-routing-与私有解析模型`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_tracks_bom_stripped_source_metadata_and_original_bytes`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_rejects_encoding_syntax_trailing_input_and_a_second_bom`

Proves:
- adapter-private loader 去除至多一个开头 UTF-8 BOM，将其余 bytes 解码为 UTF-8，并同时保留去 BOM 原文和包含 BOM 的原文件 byte size。
- 一个 JSON root 后只允许 whitespace；non-UTF-8、syntax failure、trailing non-whitespace 与第二个开头 BOM 均被拒绝。

## Case WB-JSON-MODEL-001: JSON primary document model 保留结构与源码事实

Owner: `docs/adapters/json.md#pathname-routing-与私有解析模型`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_tracks_bom_stripped_source_metadata_and_original_bytes`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_preserves_order_raw_numbers_and_source_regions`

Proves:
- mixed object/array model 记录 root kind/depth、node count 和 max depth；object member 按源码顺序保存 decoded name，array element 保持 index 顺序。
- member name、member、node 与 root source region 可还原精确的去 BOM 原文片段，包括 escaped name 的源码 spelling；number node 保留 raw decimal/exponent token，不要求算术转换。

## Case WB-JSON-DUPLICATE-001: JSON object 按 decoded member name 判重

Owner: `docs/adapters/json.md#pathname-routing-与私有解析模型`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_rejects_duplicate_decoded_member_names`

Proves:
- 同一 object 内 literal `a` 与 escaped `\u0061` 解码为相同 member name 时，adapter-private loader 将其判为 duplicate member 并拒绝文档。

## Case WB-JSON-DEPTH-001: JSON document depth 上限为 127

Owner: `docs/adapters/json.md#pathname-routing-与私有解析模型`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_accepts_depth_127_and_rejects_depth_128`

Proves:
- root depth 以 `0` 计；最大 depth `127` 的嵌套 array 成功加载并记录对应 max depth 与 node count，depth `128` 则报告配置上限 `127` 和实际 depth `128` 后拒绝。

## Case WB-JSON-REF-001: JSON ref 对特殊 object token 保持 canonical roundtrip

Owner: `docs/adapters/json.md#json-ref-grammar`

Entities:
- `cargo|docnav-json:lib:docnav_json|reference::tests::canonical_ref_encodes_root_and_special_tokens`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_ref_round_trips_special_object_keys`
- `smoke|core:real-json|CORE-JSON-NAV-001`

Proves:
- root、空 token、`~`、`/`、控制字符、非 ASCII 字符和 URI fragment 保留或转义字符生成约定的 canonical `json:#` ref；生成结果保持 ASCII-safe 且不含原始控制字符。
- 空 key、特殊字符 key、控制字符 key、非 ASCII key 和纯数字 object key 的生成 ref 均解析回对应 document node。
- 真实 core CLI automatic outline 返回特殊 object key 的 canonical ASCII-safe ref；显式 read 与 find-ref read 均原样保留该 ref，并读取对应 value。

## Case WB-JSON-REF-002: JSON ref 区分非法 spelling 与文档内无匹配

Owner: `docs/adapters/json.md#json-ref-grammar`

Entities:
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_ref_rejects_noncanonical_or_malformed_spelling`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_ref_classifies_context_sensitive_paths`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_maps_invalid_and_missing_refs_to_distinct_diagnostics`
- `smoke|core:real-json|CORE-JSON-FAIL-001`

Proves:
- 缺失或错误 prefix、non-root 缺少 `/`、非法或非 canonical percent/`~` escape、原始非 ASCII/control 字符和无效 UTF-8 percent bytes 均分类为 invalid ref。
- array context 只接受 canonical index token，而相同纯数字 token 在 object context 仍作为 member name；grammar canonical 但不存在、越界或不能从 scalar 继续的 path 分类为 not found，root ref 和存在的 path 正常解析。
- trait-dispatched `read` operation 把错误 prefix 与非 canonical array index 投影为带输入 ref 和 reason 的 `REF_INVALID`，把 canonical missing ref 投影为仅带输入 ref 的 `REF_NOT_FOUND`。
- 真实 core CLI read 把 noncanonical array index `01` 投影为带 ref/reason 的 `REF_INVALID`，把 canonical missing index `9` 投影为保留 ref 的 `REF_NOT_FOUND`。

## Case WB-JSON-TRAVERSAL-001: JSON 私有遍历形成确定性 descendant entries

Owner: `docs/adapters/json.md#outline`

Entities:
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_preserve_source_order_labels_kinds_and_canonical_refs`

Proves:
- adapter-private traversal 对 mixed tree 按 object 源码顺序和 array index 顺序形成 depth-first preorder descendants，并为每项保留对应 value kind、object decoded-name 或 array-index label，以及 canonical JSON ref；空 key 的正常 label 是两个双引号字符 `""`，ref 是 `json:#/`。

## Case WB-JSON-TRAVERSAL-002: JSON 私有遍历按 root 形态决定 entry set

Owner: `docs/adapters/json.md#outline`

Entities:
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_omit_empty_container_roots`
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_keep_a_root_scalar_navigable`

Proves:
- empty object 与 empty array root 不形成 traversal entry；root string scalar 形成唯一 `json:#`、`<root>`、string-kind entry。

## Case WB-JSON-CONTENT-001: JSON 私有 structured content 保留确定性 spelling 与完整 cost

Owner: `docs/adapters/json.md#read`

Entities:
- `cargo|docnav-json:lib:docnav_json|content::tests::structured_root_preserves_source_order_raw_numbers_and_full_cost`
- `cargo|docnav-json:lib:docnav_json|content::tests::structured_selected_values_use_pinned_scalar_escaping_without_trailing_newline`

Proves:
- root 与 selected nested value 的 adapter-private structured content 使用两空格 layout、object 源码顺序、原始 number token，以及 workspace-pinned serializer 的 string escape、Unicode 和无尾随换行 spelling。
- structured content 的 selection-scoped `lines`、`bytes`、`tokens` cost 均针对分页前完整 content 计算。

## Case WB-JSON-CONTENT-002: JSON full-read facts 保留去 BOM 原文与实际 cost

Owner: `docs/adapters/json.md#info-与-full-read`

Entities:
- `cargo|docnav-json:lib:docnav_json|content::tests::full_read_strips_one_bom_only_and_measures_the_actual_source`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::full_read_hooks_preserve_bom_stripped_source_and_measure_actual_cost`

Proves:
- adapter-private full-read facts 去除一个可选 UTF-8 BOM 后保留 source spelling、换行与外围 whitespace，并对实际返回 text 计算 selection-scoped `lines`、`bytes`、`tokens` cost。
- direct full-read call 返回同一 BOM-stripped source 与 `application/json`；requested-unit cost measurement 从实际返回 text 的完整 measurements 中稳定筛选。

## Case WB-JSON-INFO-001: JSON Adapter info strategy 返回 document、adapter 与 tree metadata

Owner: `docs/adapters/json.md#info-与-full-read`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::info_reports_exact_bom_aware_document_and_nested_metadata`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::info_reports_every_root_kind_with_root_depth_zero`

Proves:
- trait-dispatched `info` operation 返回 `application/json`、`UTF-8`、包含可选 BOM 的原文件 byte size、固定 JSON adapter/format identity，以及 key set 精确为 `{root_kind, node_count, max_depth}` 的 metadata。
- 六种 JSON root kind 均使用稳定名称；root-only document 的 node count 为一、max depth 为零，nested document 的 node count 与 max depth 反映完整 tree。

## Case WB-JSON-PAGING-001: JSON 私有 text paging 保持 Unicode content 与完整 cost

Owner: `docs/adapters/json.md#paginationcost-与截断`

Entities:
- `cargo|docnav-json:lib:docnav_json|paging::tests::text_pages_reassemble_unicode_scalar_content_and_keep_full_cost`
- `cargo|docnav-json:lib:docnav_json|paging::tests::empty_and_past_end_pages_are_terminal`

Proves:
- adapter-private text paging 不拆分 Unicode scalar，连续 page 单调前进且可重组完整 structured content；每页保留分页前完整 content 的 cost，超过末页时返回空 content 和 terminal page。

## Case WB-JSON-PAGING-002: JSON entry paging 保留完整 ref 并持续前进

Owner: `docs/adapters/json.md#paginationcost-与截断`

Entities:
- `cargo|docnav-json:lib:docnav_json|paging::tests::entry_pages_with_tiny_limit_preserve_long_refs_and_make_progress`
- `cargo|docnav-json:lib:docnav_json|paging::tests::empty_and_past_end_pages_are_terminal`
- `cargo|docnav-json:lib:docnav_json|paging::tests::find_entry_pages_preserve_occurrences_facts_and_terminal_semantics`
- `cargo|docnav-json:lib:docnav_json|paging::tests::find_entry_pagination_pulls_only_the_current_page_and_lookahead`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_tiny_pages_preserve_occurrences_complete_refs_and_terminal_no_match`

Proves:
- tiny limit 下 adapter-private entry paging 保留完整 long ref、value kind 和原始 entry 顺序，预算截断后没有可见正常 label 内容时使用最小非空 fallback `.`，并使 next page 单调前进；空集合与超过末页请求返回空 entries 和 terminal page。
- adapter-private find entry paging 在足额预算下保留完整 facts；tiny limit 下按 source occurrence 顺序持续前进，保留 same-ref 重复项、完整 ref 与 location，预算截断后没有可见正常 label 内容时只将 label 压缩为最小非空 fallback `.`，空集合和超过末页请求保持 terminal。
- find entry pagination 的 retained entry working set 随请求的 page limit 保持有界，continuation 判定只增加有界状态，不在分页前保留完整 match set。
- trait-dispatched `find` operation 在 tiny limit 下逐页返回每个 occurrence，保留完整 ref 和最小非空 fallback label，page 单调前进；结果耗尽、超过末页或 query 无命中时返回空 terminal matches。

## Case WB-JSON-FIND-001: JSON find 保留原文 literal occurrences

Owner: `docs/adapters/json.md#find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::literal_occurrences_are_bom_stripped_case_sensitive_non_overlapping_and_preserved`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_projects_mixed_occurrences_and_round_trips_every_ref_through_read`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_tiny_pages_preserve_occurrences_complete_refs_and_terminal_no_match`

Proves:
- adapter-private scanner 在 BOM-stripped source 上执行大小写敏感、从左到右、非重叠 literal search，保留每个 source range；多个 occurrence 映射到同一 ref 时仍分别返回，大小写不同或不存在的 query 不产生 occurrence。
- trait-dispatched `find` operation 按 source order 投影每个 occurrence，保留 same-ref 重复 match；无命中 query 返回空 terminal result。

## Case WB-JSON-FIND-002: JSON source regions 把 occurrence 归属到最深 readable ref

Owner: `docs/adapters/json.md#find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::source_regions_map_occurrences_to_the_deepest_canonical_readable_ref`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_projects_mixed_occurrences_and_round_trips_every_ref_through_read`

Proves:
- member name、scalar 与 member 内 whitespace 归属对应 child value；container structure 与 child 间 whitespace 归属最近 container；跨 child region 和 root 外围 whitespace 归属 root，原文 escaped spelling 仍映射到 decoded token 的 canonical readable ref。
- trait-dispatched `find` operation 把 key、scalar、container structure 和跨 child occurrence 投影到对应 canonical ref，并能用每个返回 ref 读取其 JSON value。

## Case WB-JSON-FIND-003: JSON find entry facts 保留 bounded label、source line 与重复 occurrence

Owner: `docs/adapters/json.md#find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::find_entries_emit_nonempty_bounded_unicode_safe_labels`
- `cargo|docnav-json:lib:docnav_json|find::tests::find_entries_keep_large_single_line_match_sets_complete_and_bounded`
- `cargo|docnav-json:lib:docnav_json|find::tests::find_label_context_scan_is_bounded_by_raw_unicode_scalars`
- `cargo|docnav-json:lib:docnav_json|find::tests::find_label_working_set_is_bounded_by_the_label_budget`
- `cargo|docnav-json:lib:docnav_json|find::tests::find_entries_preserve_repeated_matches_and_report_bom_stripped_crlf_lines`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_projects_mixed_occurrences_and_round_trips_every_ref_through_read`
- `smoke|core:real-json|CORE-JSON-NAV-001`

Proves:
- adapter-private find entry 从原文形成非空、bounded 且不切断 Unicode scalar 的 label；长行和长 query 保留命中片段并截断，Unicode whitespace 保持既有 compact 语义，空白行使用最小非空 label；每个 match 的 label construction state 和 context scan work 由 label budget 限制，context scan 不随 source line 或 occurrence 周围连续空白的长度增长。
- 同一 source region 的重复 occurrence 分别形成 entry；高频单行 occurrence set 保持完整、ref/location 一致且每项 label 有界；BOM-stripped CRLF source 的 location 使用准确的一基 source line，不存在的 query 返回空 entry set。
- trait-dispatched `find` operation 将完整 ref、`kind: "match"`、bounded source label 和一基 source line 投影为 common match facts；同 ref occurrence 保持独立，结果可序列化为符合现有 protocol schema 的 response。
- 真实 core CLI find 按 source order 返回两个 readable refs、`kind: "match"` 与 fixture source lines `5` / `8`，首个 match ref 可继续 read 到对应 value。

## Case WB-JSON-FIND-004: JSON Adapter find strategy 拒绝空 query

Owner: `docs/adapters/json.md#find`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_rejects_an_empty_query_with_the_existing_invalid_request_diagnostic`

Proves:
- trait-dispatched `find` operation 对长度为零的 query 返回 rejection，而不是成功的空 match result。
