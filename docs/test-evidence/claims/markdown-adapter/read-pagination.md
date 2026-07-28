# Claim CLAIM-MARKDOWN-READ-PAGINATION-001: Markdown read 分页按 Unicode 字符计数

Topic: `markdown-adapter`
Owner ref: `docs/adapters/markdown.md#read`

Statement:
- Markdown read pagination uses Unicode character budgets and never splits a character.

Observations:
- Markdown read pagination 按 Unicode 字符计数，不拆分字符。
- 首个分页结果通过 page metadata 暴露下一页，后续请求从对应字符边界继续。
- read cost 在分页同时仍使用 selection-scoped helper measurements。

Supported by:
- `cargo|docnav-markdown:test:adapter|paging_find::read_paginates_unicode_without_splitting_characters`
