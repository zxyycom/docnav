### Case WB-CORE-CONFIG-PATH-006: Config inspect rejects document context flags

Entry:
- `crates/docnav/src/cli/parser/tests/config_paths.rs > config_inspect_rejects_document_context_flags`

Contract:
- `docs/cli.md` 定义或约束“Core parser accepts config file path flags”所涉及的稳定行为边界。

Proves:
- 原生入口 `config_inspect_rejects_document_context_flags` 直接验证“Config inspect rejects document context flags”所描述的结果。
