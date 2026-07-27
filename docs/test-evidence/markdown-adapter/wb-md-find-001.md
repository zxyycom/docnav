### Case WB-MD-FIND-001: Markdown find ref 和 display 语义稳定

Entry:
- `crates/adapters/markdown/tests/adapter/paging_find.rs > find_ref_targets_current_visible_region_and_read_contains_match`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown find ref 和 display 语义稳定”所涉及的稳定行为边界。

Proves:
- find 匹配 hidden heading 时，ref 指向当前 visible region 或 full document fallback。
- find display 保留匹配片段且 ref 不受 display 内容影响。
- document head 命中到 `HEAD:leading` 的语义由 `WB-MD-DOCHEAD-002` 覆盖。
