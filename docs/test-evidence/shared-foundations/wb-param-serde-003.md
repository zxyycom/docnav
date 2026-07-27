### Case WB-PARAM-SERDE-003: Present null false and empty containers produce candidates

Entry:
- `crates/shared/cli-config-resolution-serde/src/tests.rs > present_null_false_and_empty_containers_produce_candidates`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“serde config-path mapping preserves candidate facts”所涉及的稳定行为边界。

Proves:
- 原生入口 `present_null_false_and_empty_containers_produce_candidates` 直接验证“Present null false and empty containers produce candidates”所描述的结果。
