### Case WB-READABLE-RENDERER-024: Pointer without leading slash fails config validation

Entry:
- `crates/shared/readable/src/renderer/tests/errors.rs > pointer_without_leading_slash_fails_config_validation`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private config/error 边界稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `pointer_without_leading_slash_fails_config_validation` 直接验证“Pointer without leading slash fails config validation”所描述的结果。
