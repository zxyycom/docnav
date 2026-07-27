### Case WB-CORE-ARGS-011: Auto read modes keep the canonical identity and exact tokens

Entry:
- `crates/docnav/src/cli/parser/tests/document_arguments/values.rs > auto_read_modes_keep_the_canonical_identity_and_exact_tokens`

Contract:
- `docs/cli.md` 定义或约束“Core parser 保持 operation 参数所有权”所涉及的稳定行为边界。

Proves:
- 原生入口 `auto_read_modes_keep_the_canonical_identity_and_exact_tokens` 直接验证“Auto read modes keep the canonical identity and exact tokens”所描述的结果。
