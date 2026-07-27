### Case WB-OUTPUT-DOCUMENT-005: Writer failure after rendering stays a writer error

Entry:
- `crates/shared/output/src/tests.rs > writer_failure_after_rendering_stays_a_writer_error`

Contract:
- `docs/output.md` 定义或约束“共享 document output facade 分层”所涉及的稳定行为边界。

Proves:
- 原生入口 `writer_failure_after_rendering_stays_a_writer_error` 直接验证“Writer failure after rendering stays a writer error”所描述的结果。
