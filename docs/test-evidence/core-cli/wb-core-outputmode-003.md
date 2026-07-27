### Case WB-CORE-OUTPUTMODE-003: Parse explicit protocol json

Entry:
- `crates/docnav/src/cli/parser/tests/output.rs > parse_explicit_protocol_json`

Contract:
- `docs/output.md` 定义或约束“Core parser document output mode 解析稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `parse_explicit_protocol_json` 直接验证“Parse explicit protocol json”所描述的结果。
