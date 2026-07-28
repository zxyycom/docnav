# Claim CLAIM-MARKDOWN-FIND-VISIBLE-REGION-001: Markdown find ref 和 display 语义稳定

Topic: `markdown-adapter`
Owner ref: `docs/adapters/markdown.md#find`

Statement:
- A find hit below a hidden heading returns a ref for the current visible region or the full-document fallback.

Observations:
- find 匹配 hidden heading 时，ref 指向当前 visible region 或 full document fallback。
- find display 保留匹配片段且 ref 不受 display 内容影响。

Supported by:
- `cargo|docnav-markdown:test:adapter|paging_find::find_ref_targets_current_visible_region_and_read_contains_match`
- `cargo|docnav-markdown:test:adapter|paging_find::find_falls_back_to_full_document_when_no_heading_is_visible`
