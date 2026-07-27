# Claim CLAIM-WB-MD-PAGE-001: Markdown read 分页按 Unicode 字符计数

Topic: `markdown-adapter`
Owner ref: `docs/adapters/markdown.md#read`

Statement:
- Markdown read pagination uses Unicode character budgets and never splits a character.

Observations:
- Markdown read pagination 按 Unicode 字符计数，不拆分字符。
- page 前进和结束状态可通过返回的 page metadata 观察。
- read cost 使用 selection-scoped helper measurements；token cost 不参与分页预算。

Supported by:
- `cargo|docnav-markdown:test:adapter|paging_find::read_paginates_unicode_without_splitting_characters`
