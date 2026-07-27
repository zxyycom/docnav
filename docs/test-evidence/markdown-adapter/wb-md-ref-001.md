### Case WB-MD-REF-001: Markdown 重复标题生成唯一可读 ref

Entry:
- `crates/adapters/markdown/tests/adapter/outline_ref.rs > duplicate_heading_paths_generate_unique_refs_and_read_unique_sections`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown 重复标题生成唯一可读 ref”所涉及的稳定行为边界。

Proves:
- 位于不同结构坐标的重复 heading 会生成唯一 ref，且每个 ref 都能读取对应 section。
