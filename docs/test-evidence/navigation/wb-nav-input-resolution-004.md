### Case WB-NAV-INPUT-RESOLUTION-004: Pagination disabled normalizes protocol and standard input limit

Entry:
- `crates/shared/navigation/src/tests/navigation/native_options/resolution.rs > pagination_disabled_normalizes_protocol_and_standard_input_limit`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation input resolution 保持来源解析边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `pagination_disabled_normalizes_protocol_and_standard_input_limit` 直接验证“Pagination disabled normalizes protocol and standard input limit”所描述的结果。
