### Case WB-CORE-OUTPUT-010: Rendered writer failure stays an io failure

Entry:
- `crates/docnav/src/output/tests.rs > rendered_writer_failure_stays_an_io_failure`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `rendered_writer_failure_stays_an_io_failure` 直接验证“Rendered writer failure stays an io failure”所描述的结果。
