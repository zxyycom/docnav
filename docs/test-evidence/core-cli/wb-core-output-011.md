### Case WB-CORE-OUTPUT-011: Built in render failure uses existing core error id

Entry:
- `crates/docnav/src/output/tests.rs > built_in_render_failure_uses_existing_core_error_id`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `built_in_render_failure_uses_existing_core_error_id` 直接验证“Built in render failure uses existing core error id”所描述的结果。
