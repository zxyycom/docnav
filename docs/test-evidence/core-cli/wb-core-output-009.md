### Case WB-CORE-OUTPUT-009: Readable view renderer fatal uses bounded stderr and internal exit

Entry:
- `crates/docnav/src/output/tests.rs > readable_view_renderer_fatal_uses_bounded_stderr_and_internal_exit`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `readable_view_renderer_fatal_uses_bounded_stderr_and_internal_exit` 直接验证“Readable view renderer fatal uses bounded stderr and internal exit”所描述的结果。
