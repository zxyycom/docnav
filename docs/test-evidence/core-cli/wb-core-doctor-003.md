### Case WB-CORE-DOCTOR-003: Adapter layer failure dominates multiple doctor failures

Entry:
- `crates/docnav/src/config/doctor.rs > adapter_layer_failure_dominates_multiple_doctor_failures`

Contract:
- `docs/cli.md` 定义或约束“Doctor 聚合 typed check 退出码”所涉及的稳定行为边界。

Proves:
- 原生入口 `adapter_layer_failure_dominates_multiple_doctor_failures` 直接验证“Adapter layer failure dominates multiple doctor failures”所描述的结果。
