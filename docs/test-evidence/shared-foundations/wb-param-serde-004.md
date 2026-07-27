### Case WB-PARAM-SERDE-004: Missing path or non object intermediate produces no candidate

Entry:
- `crates/shared/cli-config-resolution-serde/src/tests.rs > missing_path_or_non_object_intermediate_produces_no_candidate`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“serde config-path mapping preserves candidate facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `missing_path_or_non_object_intermediate_produces_no_candidate` 直接验证“Missing path or non object intermediate produces no candidate”所描述的结果。
