# adapter-contract Specification

## Purpose
Define linked adapter interface boundaries: static descriptors, manifest/probe metadata, adapter-owned native option declarations, operation handler inputs, structured operation results, adapter diagnostics, and optional full-read support hooks. `protocol-contract` owns raw envelopes; `output-contract` owns public output rendering.

## Requirements
### Requirement: Linked adapter handlers receive prepared operation input
Linked adapter handlers MUST receive operation-specific typed input after core CLI parsing, config source loading, adapter selection, native option extraction, default resolution, and request construction have completed.

#### Scenario: Handler receives outline input
- **WHEN** navigation dispatches an outline operation to a selected adapter
- **THEN** the handler receives the normalized document path
- **THEN** it receives typed outline arguments and selected adapter options
- **THEN** raw CLI argv and raw config file parsing are already complete

#### Scenario: Handler receives invalid caller intent
- **WHEN** caller input is invalid before adapter dispatch
- **THEN** navigation or the owning input boundary reports the diagnostic
- **THEN** the linked adapter handler is not invoked for that invalid request

### Requirement: Adapter metadata excludes implementation source
Adapter manifest, probe, and descriptor metadata MUST describe adapter identity, supported format facts, native option declarations, and operation support. Adapter implementation source MUST remain a core static-registry fact instead of manifest/probe metadata.

#### Scenario: Core lists built-in adapters
- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest/probe metadata describes adapter capability and format support only

### Requirement: Native options are adapter-owned declarations
Format-native options MUST be declared by the owning adapter and consumed by navigation input resolution as owner-scoped input sources. Shared layers MUST accept native option input only through selected-adapter declarations.

#### Scenario: Adapter declares a native option
- **WHEN** a Markdown adapter option is registered in the static registry
- **THEN** navigation can extract and validate that option for Markdown operations
- **THEN** the option applies only to the declaring adapter

#### Scenario: Caller supplies an undeclared option
- **WHEN** caller input contains a native option not declared for the selected adapter
- **THEN** input resolution reports a strict caller-input diagnostic
- **THEN** dispatch stops before forwarding the unknown option

### Requirement: Adapter results preserve format semantics
Adapters MUST return structured operation results or adapter diagnostics that preserve format-owned facts such as refs, content type, parse boundaries, cost facts, and operation-specific item metadata. Core and output layers MUST project those facts without replacing adapter semantics.

#### Scenario: Adapter returns read content
- **WHEN** a linked adapter returns read content with `content_type`
- **THEN** core and output surfaces preserve that content type
- **THEN** display rendering may summarize the content without changing its machine facts

### Requirement: Adapter operation support is explicit
Adapter descriptors MUST declare supported document operations and any optional hook sets, including unstructured full-read support and cost measurements used by navigation pre-dispatch policy. Navigation uses only declared support facts when selecting adapter-level hooks.

#### Scenario: Adapter supports unstructured full read
- **WHEN** an adapter declares a full-read hook set
- **THEN** navigation may use that declaration for opt-in full-read pre-dispatch
- **THEN** the adapter still owns the content and cost facts it returns

#### Scenario: Adapter lacks a required hook
- **WHEN** policy requires a hook that the selected adapter does not declare
- **THEN** navigation reports the unsupported boundary
- **THEN** fallback behavior must come from a declared owner rather than inference
