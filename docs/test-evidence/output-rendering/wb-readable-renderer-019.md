### Case WB-READABLE-RENDERER-019: Header json is valid standalone

Entry:
- `crates/shared/readable/src/renderer/tests/success.rs > header_json_is_valid_standalone`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private block/framing 规则”所涉及的稳定行为边界。

Proves:
- 原生入口 `header_json_is_valid_standalone` 直接验证“Header json is valid standalone”所描述的结果。
