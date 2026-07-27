### Case WB-MD-PAGE-001: Markdown read 分页按 Unicode 字符计数

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > read_paginates_unicode_without_splitting_characters`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown read 分页按 Unicode 字符计数”所涉及的稳定行为边界。

Proves:
- Markdown read pagination 按 Unicode 字符计数，不拆分字符。
- page 前进和结束状态可通过返回的 page metadata 观察。
- read cost 使用 selection-scoped helper measurements；token cost 不参与分页预算。
