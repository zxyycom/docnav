# Claim CLAIM-NAVIGATION-CONFIG-SOURCE-DISPATCH-001: Navigation config source loading and dispatch 稳定

Topic: `navigation`
Owner ref: `docs/navigation-input-resolution.md#docnav-navigation`

Statement:
- Navigation owns loading, validation and adapter scoping for the raw config sources handed off by core before closed-input dispatch.

Observations:
- `docnav-navigation` 接收 config source descriptor paths 并由 navigation boundary 加载 project/user raw config sources。
- Project config source values under `options.<selected-adapter-id>.<option-key>` participate in selected catalog resolution and closed-input dispatch, producing the expected protocol success result.
- Values under other known adapter id namespaces remain separate source facts and are not forwarded to the selected strategy.
- Nested non-object config source shapes at `defaults`、`defaults.pagination` and `options` return navigation-owned typed input errors.

Supported by:
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::navigation_loads_project_and_user_config_sources_from_descriptors`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::config_sources::navigation_rejects_nested_non_object_config_shapes`
- `cargo|docnav-navigation:lib:docnav_navigation|tests::navigation::native_options::adapter_scopes::navigation_does_not_forward_other_known_adapter_namespace`
