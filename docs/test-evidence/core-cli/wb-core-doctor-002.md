### Case WB-CORE-DOCTOR-002: Doctor reports explicit missing config as failure

Entry:
- `crates/docnav/src/config/doctor.rs > doctor_reports_explicit_missing_config_as_failure`

Contract:
- `docs/cli.md` 定义或约束“Doctor 聚合 typed check 退出码”所涉及的稳定行为边界。

Proves:
- 原生入口 `doctor_reports_explicit_missing_config_as_failure` 直接验证“Doctor reports explicit missing config as failure”所描述的结果。
