### Case WB-READABLE-RENDERER-022: Non string target fails

Entry:
- `crates/shared/readable/src/renderer/tests/errors.rs > non_string_target_fails`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private config/error 边界稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `non_string_target_fails` 直接验证“Non string target fails”所描述的结果。
