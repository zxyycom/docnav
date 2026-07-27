### Case WB-READABLE-RENDERER-021: Pointer missing from value fails

Entry:
- `crates/shared/readable/src/renderer/tests/errors.rs > pointer_missing_from_value_fails`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private config/error 边界稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `pointer_missing_from_value_fails` 直接验证“Pointer missing from value fails”所描述的结果。
