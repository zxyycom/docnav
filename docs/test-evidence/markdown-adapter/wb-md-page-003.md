### Case WB-MD-PAGE-003: Outline paginates with response page until end and past end

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > outline_paginates_with_response_page_until_end_and_past_end`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown outline/find pagination 保持 continuation”所涉及的稳定行为边界。

Proves:
- 原生入口 `outline_paginates_with_response_page_until_end_and_past_end` 直接验证“Outline paginates with response page until end and past end”所描述的结果。
