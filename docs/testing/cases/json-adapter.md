# json-adapter

## Case WB-JSON-MANIFEST-001: JSON 私有 manifest 声明固定格式身份

Owner: `docs/adapters/json.md#current交付与公共边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::manifest_declares_fixed_json_identity`
- `smoke|core:real-json|CORE-JSON-NAV-001`

Proves:
- adapter-private manifest 通过语义校验，并固定声明 adapter id `docnav-json`、format id `json`、按既有顺序排列的 suffixes `.json` / `.code-workspace` / `.jsonc` / `.code-snippets` / `.jsonld` / `.geojson` / `.har` / `.webmanifest` / `.ipynb` / `.sarif`、exact filenames `.prettierrc` / `.watchmanconfig` / `Pipfile.lock` / `deno.lock` 和 content types `application/json` / `application/jsonc`；不声明其它 JSON-family hints 或 identity。
- 真实 CLI `adapter list` 从 core static registry 原样投影同一 exact descriptor 与 `implementation_source: core_static`。

## Case WB-JSON-PATHNAME-HINTS-001: JSON pathname hints 选择统一 generic navigation

Owner: `docs/adapters/json.md#current交付与公共边界`

Entities:
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::core_linked_json_supports_automatic_and_declared_selection_and_reports_selected_content_failure`
- `cargo|docnav:lib:docnav|runtime::tests::linked_adapter::selected_json_uses_only_common_closed_inputs_and_excludes_markdown_native_option`
- `smoke|core:real-json|CORE-JSON-NAV-001`
- `smoke|core:real-json|CORE-JSON-FAIL-001`

Proves:
- 九个新增 complete-basename hints 逐一通过 static registry automatic selection 进入同一个 linked `docnav-json` definition，而不创建新的 adapter、format 或 grammar identity。
- 代表性 `.jsonld` suffix 与 `Pipfile.lock` exact filename 均可完成真实 `outline -> ref -> read`，复用同一 JSONC-capable grammar、canonical ref 与 generic structural navigation；结果不声明 pathname-specific profile semantics。
- 新增 `.sarif` hint 命中的 grammar-invalid document 返回 adapter-owned `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`，不会 fallback 到其它 adapter。
- Automatic `.jsonld` selection 只把 closed common page/limit input 交给 JSON strategy；matched suffix、format identity 与 Markdown-only native option 不进入 selected input，core public parameter inventory 保持独立既有 Case 的 exact set。

## Case WB-JSON-SELECTED-PARSE-001: Selected JSON strategy parses the actual document

Owner: `docs/adapter-contract.md#文档操作执行边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::selected_outline_parses_actual_document_independently_of_path_hint`

Proves:
- Once the JSON definition is selected, definition dispatch parses the actual BOM-prefixed strict JSON document and returns JSON-owned outline facts even when the pathname itself supplies no JSON hint.

## Case WB-JSON-OUTLINE-001: JSON Adapter outline strategy 投影 common entries 与有限分页

Owner: `docs/adapters/json.md#currentoutline`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_projects_mixed_json_to_exact_common_entries`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_handles_empty_container_roots_and_root_scalar`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_tiny_pages_preserve_complete_refs_and_terminate`

Proves:
- trait-dispatched `outline` operation 把 mixed JSON 按确定性 preorder 投影为只含 common fields 的 structured entries，保留完整 canonical ref、decoded object label、array index label 和六种 value kind；空 key 的正常 label 是两个双引号字符 `""` 且 ref 是 `json:#/`，Unicode key 保留 decoded label。
- empty container root 返回空 terminal entries，root scalar 返回唯一 root entry；tiny limit 下完整 ref 与 kind 保留，预算截断后没有可见正常 label 内容时只把 label 压缩为最小非空 fallback `.`，page 单调前进，结果耗尽及超过末页时 terminal。

## Case WB-JSON-READ-001: JSON Adapter read strategy 保留 selected spelling、ref、cost 与 Unicode pagination

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_round_trips_outline_refs_and_formats_selected_values`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_paginates_unicode_and_keeps_complete_cost_on_every_page`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_projects_base_and_comment_views_from_only_the_selected_frame`

Proves:
- trait-dispatched `read` operation 接受 outline 返回的 refs，并保留输入 ref 与 `application/json` content type；selected root、object、empty container、string、boolean 和 root scalar 使用确定性 structured spelling，number 保留原始 source token。
- Unicode-safe pagination 不切断 scalar，page 单调前进并可重组完整 selected content；每页及超过末页请求均保留分页前完整 cost，结果耗尽时 terminal。
- base、direct-comment与tail-comment ref在同一 selected-first resolution 上分别投影 strict JSON 或 exact source-order comment tokens（每个 token 后 LF）加同一 normalized value；root、empty-key和array-index selection 的 ancestor bundles 不泄漏，comment views使用 `application/jsonc`，其分页与完整 projection cost保持既有语义。

## Case WB-JSON-DIAGNOSTICS-001: Selected JSON content failures use stable document diagnostics

Owner: `docs/adapters/json.md#currentinfofull-read安全与-diagnostics`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::selected_outline_maps_current_document_failures_to_stable_diagnostics`
- `smoke|core:real-json|CORE-JSON-FAIL-001`

Proves:
- A selected JSON outline observes the document view it actually opens; a missing file and invalid UTF-8 preserve the existing exact document diagnostics, while other path-access failures use a stable reason without exposing the operating-system attachment.
- Unterminated comments、JSON5 extensions、missing/doubled commas and root `{,}` / `[,]` use `JSON_SYNTAX_INVALID`; trailing non-trivia、a second root and complete-root后的 `1 {,}` / `1 [,]` use `JSON_TRAILING_INPUT`; duplicate decoded member and depth overflow retain their dedicated stable reasons.
- Every selected content failure returns `DOCUMENT_CONTENT_INVALID` details containing only the normalized path and stable reason, without parser messages/types、offsets、duplicate names or internal attachments.
- Real CLI explicit `--adapter docnav-json` selection on non-JSON `.md` paths returns all four selected JSON `DOCUMENT_CONTENT_INVALID` reasons above；automatic `.sarif` selection 的 representative syntax failure 使用同一 JSON-owned diagnostic，二者都不 fallback 到另一 adapter。

## Case WB-JSON-PARSE-001: JSON loader 限定完整 UTF-8 单值输入

Owner: `docs/adapters/json.md#currentgrammar-与私有-source-model`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_tracks_bom_stripped_source_metadata_and_original_bytes`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_rejects_encoding_syntax_trailing_input_and_a_second_bom`

Proves:
- adapter-private loader 去除至多一个开头 UTF-8 BOM，将其余 bytes 解码为 UTF-8，并同时保留去 BOM 原文和包含 BOM 的原文件 byte size。
- 一个 JSON root 后只允许 whitespace；non-UTF-8、syntax failure、trailing non-whitespace 与第二个开头 BOM 均被拒绝。

## Case WB-JSONC-LOADER-001: JSONC loader 接受闭合 grammar 且保留 primary model 事实

Owner: `docs/adapters/json.md#currentgrammar-与私有-source-model`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_accepts_closed_jsonc_grammar_and_preserves_primary_model_facts`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_accepts_comment_line_endings_and_rejects_syntax_outside_closed_grammar`

Proves:
- adapter-private loader 接受可选 UTF-8 BOM、`//` / `/*...*/` comments，以及每个非空 object 或 array 一个 trailing comma，不把此行为扩展为其它 JSON5 syntax。
- 加载后的 primary model 保留精确 BOM-stripped source、原始 byte size、logical object/array structure、raw number token 与 source regions，因而后续 scanner/model 实现不能以第二棵 logical tree 换取 JSONC acceptance。
- line comments在 LF、CRLF、lone CR或 EOF结束，block comments在首个 `*/`结束；JSON5、missing/doubled comma、root `{,}` / `[,]`、unterminated comment与multiple roots仍被闭合 grammar拒绝，而complete-root后的 `1 {,}` / `1 [,]`按 trailing input 拒绝。

## Case WB-JSONC-ATTRIBUTION-001: JSONC comments 按binding或tail唯一归属

Owner: `docs/adapters/json.md#currentcomment-attribution`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_accepts_comment_line_endings_and_rejects_syntax_outside_closed_grammar`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_attributes_direct_empty_and_tail_comments_once_in_source_order`

Proves:
- root、object member与array index的leading/header/suffix/same-line comments进入各自direct bundle，空容器内部comments进入container自身direct bundle，独立后续行comments进入所在container tail。
- LF、CRLF、lone CR和EOF都按lexical-line placement确定previous/next direct binding；delimiter-only direct与tail comment仍形成present non-empty bundle。
- root internal tail与document tail合并为一个source-ordered、可非连续bundle；每个exact raw comment span恰好出现一次，direct/tail互斥，`None`表示absent而`Some`始终非空且index递增。

## Case WB-JSONC-BOUNDS-001: JSONC scanner和attribution保持有界工作与深度

Owner: `docs/adapters/json.md#current验证边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_keeps_jsonc_depth_and_comment_evidence_bounded_for_hostile_input`
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_visit_each_comment_bundle_item_once_on_a_wide_comment_corpus`
- `cargo|docnav-json:lib:docnav_json|content::tests::comment_projection_visits_only_the_selected_bundle_on_a_wide_comment_corpus`

Proves:
- JSONC source仍接受maximum depth 127并拒绝128，wide/deep hostile corpus只保留与source bytes和comment count成比例的scanner/model evidence。
- scanner cursor单调覆盖source一次，attribution每个comment只落入一个slot一次，不使用time threshold证明复杂度。
- 在同一个 1,024-item、每项一条 direct comment 的 corpus 上，outline 的 summary 访问总数恰为 `N`；direct-comment read 只两次访问所选 bundle（预分配与复制），而非按 item 或 comment set 重扫。

## Case WB-JSON-MODEL-001: JSON primary document model 保留结构与源码事实

Owner: `docs/adapters/json.md#currentgrammar-与私有-source-model`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_tracks_bom_stripped_source_metadata_and_original_bytes`
- `cargo|docnav-json:lib:docnav_json|document::tests::load_preserves_order_raw_numbers_and_source_regions`

Proves:
- mixed object/array model 记录 root kind/depth、node count 和 max depth；object member 按源码顺序保存 decoded name，array element 保持 index 顺序。
- member name、member、node 与 root source region 可还原精确的去 BOM 原文片段，包括 escaped name 的源码 spelling；number node 保留 raw decimal/exponent token，不要求算术转换。

## Case WB-JSON-DUPLICATE-001: JSON object 按 decoded member name 判重

Owner: `docs/adapters/json.md#currentgrammar-与私有-source-model`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_rejects_duplicate_decoded_member_names`

Proves:
- 同一 object 内 literal `a` 与 escaped `\u0061` 解码为相同 member name 时，adapter-private loader 将其判为 duplicate member 并拒绝文档。

## Case WB-JSON-DEPTH-001: JSON document depth 上限为 127

Owner: `docs/adapters/json.md#currentgrammar-与私有-source-model`

Entities:
- `cargo|docnav-json:lib:docnav_json|document::tests::load_accepts_depth_127_and_rejects_depth_128`

Proves:
- root depth 以 `0` 计；最大 depth `127` 的嵌套 array 成功加载并记录对应 max depth 与 node count，depth `128` 则报告配置上限 `127` 和实际 depth `128` 后拒绝。

## Case WB-JSON-REF-001: JSON ref 对特殊 object token 保持 canonical roundtrip

Owner: `docs/adapters/json.md#currentjson-ref-grammar-与三种-view`

Entities:
- `cargo|docnav-json:lib:docnav_json|reference::tests::canonical_ref_encodes_root_and_special_tokens`
- `cargo|docnav-json:lib:docnav_json|reference::tests::comment_ref_views_parse_and_generate_canonical_tokens`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_selection_round_trips_special_object_keys`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_selection_preserves_selected_first_binding_and_comment_context`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_comment_views_support_root_scalar_array_index_and_coexistence`
- `smoke|core:real-json|CORE-JSON-NAV-001`

Proves:
- root、空 token、`~`、`/`、控制字符、非 ASCII 字符和 URI fragment 保留或转义字符生成约定的 canonical `json:#` ref；生成结果保持 ASCII-safe 且不含原始控制字符。
- base、direct-comment与tail-comment view对同一logical path只改变固定prefix，三者均解析为明确view和相同canonical tokens；root、空key和array index refs保持无歧义。
- 空 key、特殊字符 key、控制字符 key、非 ASCII key 和纯数字 object key 的生成 ref 均解析回对应 document node。
- document resolution产生selected-first、随后parent直到root的borrowed frame chain；root、object empty-key与array index binding保持类型区分，每个frame保留本层value及optional direct/tail bundle，且同一binding的direct与tail selection可共存。
- 真实 core CLI automatic outline 返回特殊 object key 的 canonical ASCII-safe ref；显式 read 与 find-ref read 均原样保留该 ref，并读取对应 value。
- 真实 core CLI 对 comment-aware outline/find 返回的 `json:comments:` / `json:tail-comments:` refs 不作解析或改写；显式 read 与 unique-ref auto-read 原样交给同一 adapter，nested read 保留 `application/jsonc`、cost、page 和 selected comment content。

## Case WB-JSON-REF-002: JSON ref 区分非法 spelling 与文档内无匹配

Owner: `docs/adapters/json.md#currentjson-ref-grammar-与三种-view`

Entities:
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_selection_rejects_noncanonical_or_malformed_spelling`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_selection_classifies_context_sensitive_paths`
- `cargo|docnav-json:lib:docnav_json|reference::tests::resolve_comment_refs_distinguishes_invalid_spelling_from_missing_selection`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::read_maps_invalid_and_missing_refs_to_distinct_diagnostics`
- `smoke|core:real-json|CORE-JSON-FAIL-001`

Proves:
- 缺失或错误 prefix、non-root 缺少 `/`、非法或非 canonical percent/`~` escape、原始非 ASCII/control 字符和无效 UTF-8 percent bytes 均分类为 invalid ref。
- array context 只接受 canonical index token，而相同纯数字 token 在 object context 仍作为 member name；grammar canonical 但不存在、越界或不能从 scalar 继续的 path 分类为 not found，root ref 和存在的 path 正常解析。
- malformed或未知comment view、malformed pointer/token与array-context noncanonical index分类为invalid；canonical logical path缺失或当前binding/anchor没有所选direct/tail bundle分类为not found，base ref不因comment缺失失效。
- trait-dispatched `read` operation 把错误 prefix 与非 canonical array index 投影为带输入 ref 和 reason 的 `REF_INVALID`，把 canonical missing ref 投影为仅带输入 ref 的 `REF_NOT_FOUND`。
- 真实 core CLI read 把 noncanonical array index `01` 投影为带 ref/reason 的 `REF_INVALID`，把 canonical missing index `9` 投影为保留 ref 的 `REF_NOT_FOUND`。

## Case WB-JSON-TRAVERSAL-001: JSON 私有遍历形成确定性 descendant entries

Owner: `docs/adapters/json.md#currentoutline`

Entities:
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_preserve_source_order_labels_kinds_and_canonical_refs`

Proves:
- adapter-private traversal 对 mixed tree 按 object 源码顺序和 array index 顺序形成 depth-first preorder descendants，并为每项保留对应 value kind、object decoded-name 或 array-index label，以及 canonical JSON ref；空 key 的正常 label 是两个双引号字符 `""`，ref 是 `json:#/`。

## Case WB-JSON-TRAVERSAL-002: JSON 私有遍历按 root 形态决定 entry set

Owner: `docs/adapters/json.md#currentoutline`

Entities:
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_omit_empty_container_roots`
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_keep_a_root_scalar_navigable`

Proves:
- empty object 与 empty array root 不形成 traversal entry；root string scalar 形成唯一 `json:#`、`<root>`、string-kind entry。

## Case WB-JSONC-OUTLINE-001: JSONC outline 投影 direct 与 tail comment navigation

Owner: `docs/adapters/json.md#currentoutline`

Entities:
- `cargo|docnav-json:lib:docnav_json|traversal::tests::preorder_entries_insert_direct_and_tail_comment_entries_in_expanded_tree_order`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_projects_comment_refs_summaries_and_virtual_tail_entries`

Proves:
- expanded-tree traversal 仍按 logical object source order、array index order执行 preorder；有 direct bundle 的 root/member/index logical entry 使用 canonical direct-comment ref，其余 logical entry 保留 base ref。
- root object/array 仅在 root direct bundle 存在时于 descendants 前生成 `<root>` entry；空 normalized body 只省略 summary，不移除 comment ref。只有 root tail bundle 时不生成 root logical entry。
- nested tail virtual entry 位于其 anchor subtree 全部 descendants 后且早于后续 sibling/ancestor tail，root tail 最后；每项使用 canonical tail ref、`<tail comments>`、`tail_comments` 和可用 summary，并省略其它 optional entry fields。
- line/block comment bodies移除 delimiters、逐 body 折叠 Unicode whitespace并 trim，丢弃空 body后以 `; ` source-order join，summary保持单行。

## Case WB-JSONC-OUTLINE-PAGING-001: JSONC outline summary 服从 entry budget

Owner: `docs/adapters/json.md#currentoutline`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::outline_comment_summary_budget_shrinks_before_label_and_pages_forward`

Proves:
- entry budget首先在 Unicode scalar boundary截断并标记或省略 optional summary，随后才允许压缩 label；无论 limit 多小都保留完整 canonical comment ref和value kind。
- page continuation 按 expanded preorder 单调前进，summary裁剪不跳过或重复后续 entry，结果耗尽时 terminal。

## Case WB-JSON-CONTENT-001: JSON 私有 structured content 保留确定性 spelling 与完整 cost

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|content::tests::structured_root_preserves_source_order_raw_numbers_and_full_cost`
- `cargo|docnav-json:lib:docnav_json|content::tests::structured_selected_values_use_pinned_scalar_escaping_without_trailing_newline`

Proves:
- root 与 selected nested value 的 adapter-private structured content 使用两空格 layout、object 源码顺序、原始 number token，以及 workspace-pinned serializer 的 string escape、Unicode 和无尾随换行 spelling。
- structured content 的 selection-scoped `lines`、`bytes`、`tokens` cost 均针对分页前完整 content 计算。

## Case WB-JSON-CONTENT-002: JSON full-read facts 保留去 BOM 原文与实际 cost

Owner: `docs/adapters/json.md#currentinfofull-read安全与-diagnostics`

Entities:
- `cargo|docnav-json:lib:docnav_json|content::tests::full_read_strips_one_bom_only_and_measures_the_actual_source`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::full_read_hooks_preserve_bom_stripped_source_and_measure_actual_cost`

Proves:
- adapter-private full-read facts 去除一个可选 UTF-8 BOM 后精确保留 strict JSON/JSONC source spelling、comments、trailing comma、换行与外围 whitespace，并对实际返回 text 计算 selection-scoped `lines`、`bytes`、`tokens` cost。
- direct full-read call 对 strict source 和仅含 string markers 的 source 返回 `application/json`，对实际 comment 或 trailing-comma syntax 返回 `application/jsonc`；requested-unit cost measurement 从同一实际返回 text 的完整 measurements 中稳定筛选。

## Case WB-JSON-INFO-001: JSON Adapter info strategy 返回 document、adapter 与 tree metadata

Owner: `docs/adapters/json.md#currentinfofull-read安全与-diagnostics`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::info_reports_exact_bom_aware_document_and_nested_metadata`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::info_reports_every_root_kind_with_root_depth_zero`

Proves:
- trait-dispatched `info` operation 从已解析 syntax 派生 content type：strict source 与 string 内 comment/trailing-comma markers 返回 `application/json`，实际 comments 或 trailing comma 返回 `application/jsonc`；同时保持 `UTF-8`、包含可选 BOM 的原文件 byte size 和固定 `docnav-json` / `json` identity。
- Info metadata key set 精确为 `{root_kind, node_count, max_depth}`，不把 pathname hint 或 content type 当作 dialect input。
- 六种 JSON root kind 均使用稳定名称；root-only document 的 node count 为一、max depth 为零，nested document 的 node count 与 max depth 反映完整 tree。

## Case WB-JSON-PAGING-001: JSON 私有 text paging 保持 Unicode content 与完整 cost

Owner: `docs/adapters/json.md#current验证边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|paging::tests::text_pages_reassemble_unicode_scalar_content_and_keep_full_cost`
- `cargo|docnav-json:lib:docnav_json|paging::tests::empty_and_past_end_pages_are_terminal`

Proves:
- adapter-private text paging 不拆分 Unicode scalar，连续 page 单调前进且可重组完整 structured content；每页保留分页前完整 content 的 cost，超过末页时返回空 content 和 terminal page。

## Case WB-JSON-PAGING-002: JSON entry paging 保留完整 ref 并持续前进

Owner: `docs/adapters/json.md#current验证边界`

Entities:
- `cargo|docnav-json:lib:docnav_json|paging::tests::entry_pages_with_tiny_limit_preserve_long_refs_and_make_progress`
- `cargo|docnav-json:lib:docnav_json|paging::tests::empty_key_labels_survive_tiny_ref_only_and_complete_entry_budgets`
- `cargo|docnav-json:lib:docnav_json|paging::tests::empty_and_past_end_pages_are_terminal`
- `cargo|docnav-json:lib:docnav_json|paging::tests::find_entry_pages_preserve_occurrences_facts_and_terminal_semantics`
- `cargo|docnav-json:lib:docnav_json|paging::tests::find_entry_pagination_pulls_only_the_current_page_and_lookahead`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_tiny_pages_preserve_occurrences_complete_refs_and_terminal_no_match`

Proves:
- tiny limit 下 adapter-private entry paging 保留完整 long ref、value kind 和原始 entry 顺序，预算截断后没有可见正常 label 内容时使用最小非空 fallback `.`，但 base/direct-comment empty-key entry 在 tiny、ref-only、partial-label 和完整 entry budget 下都保留固定正常 label `""`；真实非空键名 `""` 仍服从普通 fallback。两类 entry 均使 next page 单调前进；空集合与超过末页请求返回空 entries 和 terminal page。
- adapter-private find entry paging 在足额预算下保留完整 facts；tiny limit 下按 source occurrence 顺序持续前进，保留 same-ref 重复项、完整 ref 与 location，预算截断后没有可见正常 label 内容时只将 label 压缩为最小非空 fallback `.`，空集合和超过末页请求保持 terminal。
- find entry pagination 的 retained entry working set 随请求的 page limit 保持有界，continuation 判定只增加有界状态，不在分页前保留完整 match set。
- trait-dispatched `find` operation 在 tiny limit 下逐页返回每个 occurrence，保留完整 ref 和最小非空 fallback label，page 单调前进；结果耗尽、超过末页或 query 无命中时返回空 terminal matches。

## Case WB-JSON-FIND-001: JSON find 保留原文 literal occurrences

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::literal_occurrences_are_bom_stripped_case_sensitive_non_overlapping_and_preserved`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_projects_mixed_occurrences_and_round_trips_every_ref_through_read`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_tiny_pages_preserve_occurrences_complete_refs_and_terminal_no_match`

Proves:
- adapter-private scanner 在 BOM-stripped source 上执行大小写敏感、从左到右、非重叠 literal search，保留每个 source range；多个 occurrence 映射到同一 ref 时仍分别返回，大小写不同或不存在的 query 不产生 occurrence。
- trait-dispatched `find` operation 按 source order 投影每个 occurrence，保留 same-ref 重复 match；无命中 query 返回空 terminal result。

## Case WB-JSON-FIND-002: JSON source regions 把 occurrence 归属到最深 readable ref

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::source_regions_map_occurrences_to_the_deepest_canonical_readable_ref`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_projects_mixed_occurrences_and_round_trips_every_ref_through_read`

Proves:
- member name、scalar 与 member 内 whitespace 归属对应 child value；container structure 与 child 间 whitespace 归属最近 container；跨 child region 和 root 外围 whitespace 归属 root，原文 escaped spelling 仍映射到 decoded token 的 canonical readable ref。
- trait-dispatched `find` operation 把 key、scalar、container structure 和跨 child occurrence 投影到对应 canonical ref，并能用每个返回 ref 读取其 JSON value。

## Case WB-JSON-FIND-003: JSON find entry facts 保留 bounded label、source line 与重复 occurrence

Owner: `docs/adapters/json.md#currentread-与-find`

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

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_rejects_an_empty_query_with_the_existing_invalid_request_diagnostic`

Proves:
- trait-dispatched `find` operation 对长度为零的 query 返回 rejection，而不是成功的空 match result。

## Case WB-JSON-FIND-005: JSONC find 将完整 comment span 映射到可读 comment view

Owner: `docs/adapters/json.md#currentread-与-find`

Entities:
- `cargo|docnav-json:lib:docnav_json|find::tests::comment_spans_override_only_wholly_contained_occurrences_and_use_source_ordered_lookup`
- `cargo|docnav-json:lib:docnav_json|adapter::tests::find_maps_comment_occurrences_to_comment_views_and_preserves_find_facts`

Proves:
- source find 仅当 occurrence 完全包含在已归属的 direct 或 tail comment token span 内时，分别返回该 navigation binding 的 canonical `json:comments:` 或 `json:tail-comments:` ref；跨出 comment span 的 occurrence 保持既有 deepest-covering base ref。
- comment、ordinary value 与 tail occurrence 依 source order 分别投影，LF location、bounded find pagination continuation 和 Unicode-safe existing label behavior保持；direct/tail match refs 可 round-trip 到 selected `application/jsonc` read projection，ordinary ref 保持 `application/json`。
- 每个 find operation 一次性从 primary tree 的既有 bundles 构造按 source-order 的轻量 lookup；单调 cursor 对每个 occurrence 只检查当前候选并至多跨过每个 comment 一次，不按 occurrence 全量扫描 comments。
