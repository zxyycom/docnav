### Case WB-NAVIGATION-DISPATCH-001: Navigation config source loading and dispatch 稳定

Entry:
- `crates/shared/navigation/src/tests/navigation/config_sources.rs > navigation_loads_project_and_user_config_sources_from_descriptors`

Contract:
- `docs/navigation-input-resolution.md` 定义或约束“Navigation config source loading and dispatch 稳定”所涉及的稳定行为边界。

Proves:
- `docnav-navigation` 接收 config source descriptor paths 并由 navigation boundary 加载 project/user raw config sources。
- Project config source values under `options.<selected-adapter-id>.<option-key>` participate in selected catalog resolution and closed-input dispatch, producing the expected protocol success result.
- Values under other known adapter id namespaces remain separate source facts and are not forwarded to the selected strategy.
- Nested non-object config source shapes at `defaults`、`defaults.pagination` and `options` return navigation-owned typed input errors.
