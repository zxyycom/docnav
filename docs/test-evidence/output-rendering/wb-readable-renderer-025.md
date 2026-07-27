### Case WB-READABLE-RENDERER-025: Render error uses stable id

Entry:
- `crates/shared/readable/src/renderer/tests/errors.rs > render_error_uses_stable_id`

Contract:
- `docs/output.md` 定义或约束“内置 readable renderer private config/error 边界稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `render_error_uses_stable_id` 直接验证“Render error uses stable id”所描述的结果。
