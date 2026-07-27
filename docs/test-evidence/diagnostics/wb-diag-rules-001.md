### Case WB-DIAG-RULES-001: Diagnostics code rules 保持稳定

Entry:
- `crates/shared/diagnostics/src/tests/code_rules.rs > diagnostic_code_rules_cover_each_variant`

Contract:
- `docs/architecture.md` 定义或约束“Diagnostics code rules 保持稳定”所涉及的稳定行为边界。

Proves:
- `DiagnosticCode::all()` exposes the current diagnostic registry, including representative protocol and boundary diagnostic codes.
- Each registry code exposes a non-empty unique stable string、non-empty details rule 和可用的 diagnostic projection route。
