### Case WB-NAV-ADAPTER-SOURCE-004: Automatic discovery all fail projects candidate failures

Entry:
- `crates/shared/navigation/src/tests/navigation/adapter_source.rs > automatic_discovery_all_fail_projects_candidate_failures`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation adapter selection 保持静态来源边界”所涉及的稳定行为边界。

Proves:
- 原生入口 `automatic_discovery_all_fail_projects_candidate_failures` 直接验证“Automatic discovery all fail projects candidate failures”所描述的结果。
