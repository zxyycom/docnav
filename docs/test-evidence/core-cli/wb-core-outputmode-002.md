### Case WB-CORE-OUTPUTMODE-002: Parse without output has none

Entry:
- `crates/docnav/src/cli/parser/tests/output.rs > parse_without_output_has_none`

Contract:
- `docs/output.md` 定义或约束“Core parser document output mode 解析稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `parse_without_output_has_none` 直接验证“Parse without output has none”所描述的结果。
