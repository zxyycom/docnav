### Case WB-MD-REF-MATCH-004: Matches rejects level mismatch

Entry:
- `crates/adapters/markdown/src/markdown/refs/tests.rs > matches_rejects_level_mismatch`

Contract:
- `docs/adapters/markdown.md` 定义或约束“Markdown parsed ref 精确匹配 heading 坐标”所涉及的稳定行为边界。

Proves:
- 原生入口 `matches_rejects_level_mismatch` 直接验证“Matches rejects level mismatch”所描述的结果。
