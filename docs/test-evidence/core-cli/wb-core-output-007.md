### Case WB-CORE-OUTPUT-007: Readable error uses document facade and exit policy stays local

Entry:
- `crates/docnav/src/output/tests.rs > readable_error_uses_document_facade_and_exit_policy_stays_local`

Contract:
- `docs/output.md` 定义或约束“Core 输出编排保持通道边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `readable_error_uses_document_facade_and_exit_policy_stays_local` 直接验证“Readable error uses document facade and exit policy stays local”所描述的结果。
