### Case WB-READABLE-RENDERER-023: Duplicate pointer in config fails

Entry:
- `crates/shared/readable/src/renderer/tests/errors.rs > duplicate_pointer_in_config_fails`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private config/error 边界稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `duplicate_pointer_in_config_fails` 直接验证“Duplicate pointer in config fails”所描述的结果。
