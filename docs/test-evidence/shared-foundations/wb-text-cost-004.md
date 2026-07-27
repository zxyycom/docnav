### Case WB-TEXT-COST-004: Token cost uses o200k base ordinary text

Entry:
- `crates/shared/text-cost/src/tests.rs > token_cost_uses_o200k_base_ordinary_text`

Contract:
- `docs/architecture.md` 定义或约束“Shared text cost helper 保持纯文本边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `token_cost_uses_o200k_base_ordinary_text` 直接验证“Token cost uses o200k base ordinary text”所描述的结果。
