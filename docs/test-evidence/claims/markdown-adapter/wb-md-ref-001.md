# Claim CLAIM-WB-MD-REF-001: Markdown 重复标题生成唯一可读 ref

Topic: `markdown-adapter`
Owner ref: `docs/adapters/markdown.md#重复-heading`

Statement:
- Duplicate headings at different structural coordinates receive unique readable refs that resolve to their own sections.

Observations:
- 位于不同结构坐标的重复 heading 会生成唯一 ref，且每个 ref 都能读取对应 section。

Supported by:
- `cargo|docnav-markdown:test:adapter|outline_ref::duplicate_heading_paths_generate_unique_refs_and_read_unique_sections`
