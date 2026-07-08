本 delta spec 修改 adapter contract，使 linked adapter 扩展面收敛为 registry-facing descriptor、高层 operation handler、内部 typed native option handoff/accessor 和 capability group；当前文档只在 `openspec/changes/streamline-adapter-definition-contract/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Linked adapter handlers receive prepared operation input
Linked adapter handlers MUST receive operation-specific typed input after core CLI parsing, config source loading, adapter selection, native option extraction, default resolution, request construction, and adapter-specific internal typed native option handoff have completed. Handler inputs MUST NOT require adapter implementations to consume raw CLI argv, raw config JSON, or untyped native option source values for basic type, requiredness, allowed-value, or range validation.

#### Scenario: Handler receives outline input
- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the handler receives the normalized document path
- **THEN** it receives typed outline arguments and selected adapter native option values through the internal dispatch boundary
- **THEN** raw CLI argv and raw config file parsing are already complete
- **THEN** basic native option type and range validation has already completed

#### Scenario: Handler receives invalid caller intent
- **WHEN** caller input is invalid before adapter dispatch
- **THEN** navigation or the owning input boundary reports the diagnostic
- **THEN** the linked adapter handler is not invoked for that invalid request

#### Scenario: Handler consumes adapter-specific option accessor
- **WHEN** a selected adapter declares a native option with a typed accessor or typed handoff binding
- **THEN** navigation resolves and validates the declared option before dispatch
- **THEN** the handler consumes the typed value through the adapter-owned accessor or handoff structure
- **THEN** the handler does not repeat basic JSON type or range validation for that option

### Requirement: Adapter metadata excludes implementation source
Adapter definition, manifest, probe, and descriptor metadata MUST describe adapter identity, supported format facts, native option declarations, capability declarations, and operation support. The adapter definition MUST be the registry-facing aggregation point for metadata, declarations, capability groups, and operation handler handles. Adapter implementation source MUST remain a core static-registry fact instead of adapter definition, manifest, or probe metadata.

#### Scenario: Core lists built-in adapters
- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest/probe metadata describes adapter capability and format support only
- **THEN** registry-facing adapter metadata is derived from the selected adapter definition

#### Scenario: Registry consumes a single adapter definition
- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition or equivalent facade for that adapter
- **THEN** identity, format descriptors, native option declarations, operation handlers, and optional capability groups are reachable from that definition
- **THEN** the registry does not reconstruct adapter-owned native option or capability semantics from unrelated hook methods

### Requirement: Native options are adapter-owned declarations
Format-native options MUST be declared by the owning adapter and consumed by navigation input resolution as owner-scoped input sources. Shared layers MUST accept native option input only through selected-adapter declarations. Declarations MUST provide enough typed metadata for navigation to resolve defaults, validate source values, and produce adapter-specific internal typed native option handoff or accessor values before handler dispatch without requiring external protocol JSON shape changes.

#### Scenario: Adapter declares a native option
- **WHEN** a Markdown adapter option is registered in the static registry through its adapter definition
- **THEN** navigation can extract and validate that option for Markdown operations
- **THEN** the option applies only to the declaring adapter
- **THEN** the internal dispatch boundary can provide the Markdown handler with a typed option value

#### Scenario: Caller supplies an undeclared option
- **WHEN** caller input contains a native option not declared for the selected adapter
- **THEN** input resolution reports a strict caller-input diagnostic
- **THEN** dispatch stops before forwarding the unknown option

#### Scenario: Native option declaration stays the semantic owner
- **WHEN** project config, user config, or explicit CLI input provides an adapter native option
- **THEN** navigation uses the selected adapter declaration to resolve and validate the value
- **THEN** the adapter declaration remains the source of value kind, default, range, and operation applicability semantics
- **THEN** core and navigation do not infer adapter-owned semantics from the config key name alone

### Requirement: Adapter operation support is explicit
Adapter definitions MUST declare supported document operations and any optional capability groups, including unstructured full-read support and cost measurements used by navigation pre-dispatch policy. Navigation uses only declared support facts when selecting adapter-level capabilities. Optional capability groups MUST aggregate related optional hooks under one declared owner boundary instead of requiring navigation to infer support from unrelated methods.

#### Scenario: Adapter supports unstructured full read
- **WHEN** an adapter declares a full-read capability group
- **THEN** navigation may use that declaration for opt-in full-read pre-dispatch
- **THEN** the adapter still owns the content and cost facts it returns
- **THEN** content, cost measurement, and result facts support are interpreted within the declared full-read capability boundary

#### Scenario: Adapter lacks a required hook
- **WHEN** policy requires a capability that the selected adapter does not declare
- **THEN** navigation reports the unsupported boundary
- **THEN** fallback behavior must come from a declared owner rather than inference

#### Scenario: Optional capability does not replace operation handlers
- **WHEN** an adapter declares an optional full-read capability group
- **THEN** the adapter still declares the required `outline`, `read`, `find`, and `info` operation handlers
- **THEN** navigation uses the optional capability only for the policy path that explicitly permits it
