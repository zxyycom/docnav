### Case WB-CORE-OUTPUTMODE-004: Removed output value remains a canonical candidate for navigation validation

Entry:
- `crates/docnav/src/cli/parser/tests/output.rs > removed_output_value_remains_a_canonical_candidate_for_navigation_validation`

Contract:
- `docs/output.md` 定义或约束“Core parser document output mode 解析稳定”所涉及的稳定行为边界。

Proves:
- 原生入口 `removed_output_value_remains_a_canonical_candidate_for_navigation_validation` 直接验证“Removed output value remains a canonical candidate for navigation validation”所描述的结果。
