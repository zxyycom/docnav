# markdown-adapter

## Case WB-MD-ADAPTER-OUTLINE-001: Markdown adapter outline 默认层级和 fallback 稳定

Owner: `docs/adapters/markdown.md#outline`

Entities:
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_falls_back_to_full_document_for_no_visible_heading`
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_is_flat_default_h1_to_h3_and_ignores_code_fences`

Proves:
- adapter outline 默认显示 H1-H3，并忽略 code fence 内 heading 和超出默认层级的 heading。
- 没有 visible heading 时 fallback 到 `doc:full`，且 read 能返回完整文档。

## Case WB-MD-DISPLAY-001: Markdown outline/find display 保留可读文本

Owner: `docs/adapters/markdown.md#item-facts-职责与截断`

Entities:
- `cargo|docnav-markdown:test:adapter|options_error_display::find_entry_contains_match_snippet`
- `cargo|docnav-markdown:test:adapter|options_error_display::outline_entries_include_heading_title`

Proves:
- outline display 包含 heading title，find display 包含 match snippet。
- display 不进入 ref，不影响 adapter-owned ref 语义。

## Case WB-MD-DOCHEAD-001: Markdown document head outline eligibility 和 raw facts 稳定

Owner: `docs/adapters/markdown.md#document-head`

Entities:
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_exposes_document_head_before_visible_headings_when_nonblank`
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_exposes_document_head_when_leading_region_is_frontmatter_only`
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_keeps_frontmatter_pseudo_heading_fence_pseudo_heading_and_hr_in_document_head`
- `cargo|docnav-markdown:test:adapter|outline_ref::outline_omits_document_head_for_empty_or_whitespace_only_prefix`

Proves:
- document head 定义为文档开头到第一个有效 Markdown heading 起点之前的原文区域，frontmatter 内伪 heading、代码围栏伪 heading 和普通 horizontal rule 不改变第一个有效 heading 判定。
- document head 非空、非纯空白且当前 structured outline 至少有一个可见 heading entry 时，outline 始终在 heading entries 前暴露 `HEAD:leading`。
- 空或纯空白 document head 不暴露 `HEAD:leading`，heading entries 的顺序和 canonical heading ref grammar 保持不变。
- 当前 outline 参数过滤后没有可见 heading entry 时，outline 保留单条 `doc:full` fallback，不只返回 document head entry。
- raw document head entry facts 使用非空 `label`、非 heading `kind`、`location.line_start` 和 `metadata.document_region = leading`；readable-only `display` 不进入 raw protocol contract。

## Case WB-MD-DOCHEAD-002: Markdown document head read/find roundtrip 和分页稳定

Owner: `docs/adapters/markdown.md#document-head-ref-读取`

Entities:
- `cargo|docnav-markdown:test:adapter|paging_find::find_falls_back_to_full_document_when_no_heading_is_visible`
- `cargo|docnav-markdown:test:adapter|paging_find::find_match_before_first_visible_heading_uses_document_head_ref`
- `cargo|docnav-markdown:test:adapter|paging_find::read_document_head_preserves_yaml_delimiters_and_leading_text`
- `cargo|docnav-markdown:test:adapter|paging_find::read_document_head_returns_original_markdown_and_paginates_unicode`

Proves:
- `read HEAD:leading` 返回 document head 原文，`content_type` 为 `text/markdown`，并保留 YAML frontmatter 起止 delimiter、注释、空行和普通前导正文。
- document head read 的 `limit` 和 `page` 使用普通 read content 分页规则，按 Unicode 字符预算分页且不拆分字符。
- find 命中 document head 且当前 structured outline 至少有一个可见 heading entry 时返回 `HEAD:leading`，使用该 ref read 可读取包含命中文本的 content。
- find 命中 document head 但当前 outline 使用 `doc:full` fallback 时，返回 ref 仍可 read 到包含命中文本的内容。

## Case WB-MD-ERROR-001: Markdown adapter document error 稳定

Owner: `docs/adapters/markdown.md#错误分类`

Entities:
- `cargo|docnav-markdown:test:adapter|options_error_display::non_utf8_document_returns_stable_encoding_error`

Proves:
- non-UTF-8 document 返回稳定 encoding error。

## Case WB-MD-FIND-001: Markdown find ref 和 display 语义稳定

Owner: `docs/adapters/markdown.md#find`

Entities:
- `cargo|docnav-markdown:test:adapter|paging_find::find_ref_targets_current_visible_region_and_read_contains_match`
- `cargo|docnav-markdown:test:adapter|paging_find::find_falls_back_to_full_document_when_no_heading_is_visible`

Proves:
- find 匹配 hidden heading 时，ref 指向当前 visible region 或 full document fallback。
- find display 保留匹配片段且 ref 不受 display 内容影响。

## Case WB-MD-META-001: Markdown manifest/probe/info 元数据稳定

Owner: `docs/adapter-contract.md#manifest-元数据`

Entities:
- `cargo|docnav-markdown:test:adapter|meta::definition_declares_manifest_and_full_read_capabilities`
- `cargo|docnav-markdown:test:adapter|meta::info_returns_markdown_summary`
- `cargo|docnav-markdown:test:adapter|meta::manifest_declares_markdown_v0_identity_and_formats`
- `cargo|docnav-markdown:test:adapter|meta::probe_returns_format_evidence_without_navigation_payload`

Proves:
- manifest 声明 Markdown v0 identity 和 format metadata，probe 返回 format evidence 而不泄漏 navigation payload。
- info 返回 Markdown summary。
- Markdown registry-facing definition exposes manifest identity、linked strategy and the declared full-read capability set.

## Case WB-MD-OPTIONS-001: Markdown standard input 控制可见粒度

Owner: `docs/adapters/markdown.md#可见性与-max_heading_level`

Entities:
- `cargo|docnav-markdown:test:adapter|options_error_display::adapter_owned_options_shape_outline_and_find_granularity`
- `cargo|docnav-markdown:test:adapter|options_error_display::outline_does_not_default_a_missing_max_heading_level`
- `cargo|docnav-markdown:test:adapter|options_error_display::outline_rejects_out_of_range_max_heading_level_at_adapter_boundary`

Proves:
- Closed `OutlineInput` / `FindInput` 中的 `max_heading_level` 同时影响 outline 和 find 的 visible heading granularity。
- Markdown strategy 不为缺失的 `max_heading_level` 重复提供 catalog default。
- Markdown adapter owns the `1..6` semantic range check at its strategy boundary and returns an adapter-option diagnostic for out-of-range standard input.

## Case WB-MD-OUTLINE-001: Markdown outline ref 和 display 语义稳定

Owner: `docs/adapters/markdown.md#outline`

Entities:
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::outline_entry_handles_whitespace_only_title`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::outline_entry_includes_title_level_and_cost`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::outline_generates_canonical_heading_refs`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::outline_refs_consistent_under_different_max_heading_level`

Proves:
- outline 生成 canonical ref，重复 title/path 不影响 ref，max heading level 只影响可见性。
- outline cost 按 `lines`、`bytes`、`tokens` 顺序报告 entry-scoped measurements，display 保留 title/cost，但 ref 不包含展示文本。

## Case WB-MD-PAGE-001: Markdown read 分页按 Unicode 字符计数

Owner: `docs/adapters/markdown.md#read`

Entities:
- `cargo|docnav-markdown:test:adapter|paging_find::read_paginates_unicode_without_splitting_characters`

Proves:
- Markdown read pagination 按 Unicode 字符计数，不拆分字符。
- 首个分页结果通过 page metadata 暴露下一页，后续请求从对应字符边界继续。
- read cost 在分页同时仍使用 selection-scoped helper measurements。

## Case WB-MD-PAGE-002: Markdown outline/find pagination 保持 continuation

Owner: `docs/protocol.md#分页模型`

Entities:
- `cargo|docnav-markdown:test:adapter|paging_find::find_paginates_with_response_page_until_end_and_past_end`
- `cargo|docnav-markdown:test:adapter|paging_find::outline_paginates_with_response_page_until_end_and_past_end`

Proves:
- outline 和 find 结果按 response page 继续读取直到结束。
- past-end page 返回空结果且不产生 continuation。

## Case WB-MD-PAGING-DISPLAY-001: Markdown paging helper 保留 ref 并截断 display

Owner: `docs/adapters/markdown.md#截断规则`

Entities:
- `cargo|docnav-markdown:lib:docnav_markdown|paging::tests::entry_paging_preserves_ref_and_truncates_display`

Proves:
- display 预算不足时截断 display 而不截断 ref，并在有空间时保留 ellipsis marker。

## Case WB-MD-PARSE-001: Markdown parser 忽略非 heading 结构

Owner: `docs/adapters/markdown.md#heading-识别与-section-范围`

Entities:
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::frontmatter_is_excluded_from_outline_headings`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::parser_ignores_code_fence_pseudo_heading_and_invalid_heading`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::tests::read_section_ends_at_next_same_or_higher_heading`

Proves:
- code fence pseudo heading、invalid heading 和 frontmatter 不进入 heading model。
- section boundary 按 Markdown heading 层级截断。

## Case WB-MD-REF-001: Markdown 重复标题生成唯一可读 ref

Owner: `docs/adapters/markdown.md#同一解析结果中的唯一性`

Entities:
- `cargo|docnav-markdown:test:adapter|outline_ref::duplicate_heading_paths_generate_unique_refs_and_read_unique_sections`

Proves:
- 位于不同结构坐标的重复 heading 会生成唯一 ref，且每个 ref 都能读取对应 section。

## Case WB-MD-REF-002: Markdown ref 错误区分 invalid 和 not-found

Owner: `docs/adapters/markdown.md#错误分类`

Entities:
- `cargo|docnav-markdown:test:adapter|outline_ref::read_reports_ref_invalid_for_grammar_outside_refs`
- `cargo|docnav-markdown:test:adapter|outline_ref::read_reports_ref_not_found_for_canonical_no_match`
- `cargo|docnav-markdown:test:adapter|outline_ref::structure_snapshot_ref_is_evaluated_against_current_document`

Proves:
- grammar 外输入返回 `REF_INVALID`。
- 符合 canonical grammar 且当前结构缺少匹配 section 的 ref 返回 `REF_NOT_FOUND`。
- 文档结构变化后的先前 ref 由当前结构重新判定。

## Case WB-MD-REF-GRAMMAR-001: Markdown ref grammar 稳定

Owner: `docs/adapters/markdown.md#heading-ref-grammar`

Entities:
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::refs::tests::canonical_heading_ref_uses_structural_coordinates`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::refs::tests::parse_canonical_heading_ref`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::refs::tests::parse_rejects_one_representative_per_invalid_grammar_type`

Proves:
- canonical heading ref 由 line 和 level 结构字段构成。
- parser 将一个非法字段、一个未知 ref 类型和一个前导零输入代表映射到 grammar 外输入；同类拼写和完整正则约束由 Markdown owner 的 Manual CR 维护。

## Case WB-MD-REF-MATCH-001: Markdown parsed ref 精确匹配 heading 坐标

Owner: `docs/adapters/markdown.md#heading-ref-读取`

Entities:
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::refs::tests::matches_exact_line_level`
- `cargo|docnav-markdown:lib:docnav_markdown|markdown::refs::tests::matches_rejects_coordinate_mismatch`

Proves:
- parsed heading ref 在 line 和 level 同时匹配时命中目标 heading。
- matcher 的命中条件由 parsed ref 的 line 和 level 决定。
