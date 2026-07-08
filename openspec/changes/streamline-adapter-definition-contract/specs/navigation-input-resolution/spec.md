本 delta spec 修改 navigation input resolution，使 navigation 从 adapter definition 消费 selected adapter declarations 并通过内部 dispatch boundary 向 handler 交付 typed native option values；当前文档只在 `openspec/changes/streamline-adapter-definition-contract/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Navigation selects adapter before adapter parameter extraction
Navigation MUST select the adapter using routing inputs and registry facts before extracting selected adapter native options. The selected registry entry MUST expose an adapter definition or equivalent facade, and navigation MUST consume native option declarations and capability declarations from that selected definition rather than from unselected adapters.

#### Scenario: Multiple adapters exist
- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only the selected adapter's native declarations are used for extraction
- **THEN** declarations from unselected adapter definitions remain outside the operation field set

#### Scenario: Selected definition provides capability facts
- **WHEN** navigation has selected an adapter
- **THEN** navigation reads optional capability declarations from the selected adapter definition
- **THEN** pre-dispatch policy uses only those declared support facts

### Requirement: Selected adapter declarations own parameter facts
Selected adapter typed-field declarations MUST provide adapter-owned option identity, extraction metadata, defaults, validation facts, operation applicability, and internal typed handoff/accessor binding metadata used during navigation resolution. These declarations MUST come from the selected adapter definition or equivalent facade.

#### Scenario: Selected Markdown adapter
- **WHEN** Markdown is selected
- **THEN** Markdown native option declarations are registered for extraction from the selected adapter definition
- **THEN** non-Markdown option declarations remain outside the selected declaration set
- **THEN** resolved Markdown option values can be handed to the Markdown handler as typed native option values through the internal dispatch boundary

#### Scenario: Selected adapter declaration binds typed handoff
- **WHEN** a selected adapter declaration includes typed handoff or accessor binding metadata
- **THEN** navigation uses that metadata after validation and extraction
- **THEN** request construction or dispatch prepares the adapter-specific typed option value for the handler

### Requirement: Adapter native options are owner-scoped
Navigation MUST validate and extract native options only when they are declared by the selected adapter. Undeclared owner-scoped options MUST fail strictly. Declared native options MUST be resolved into typed values before dispatch so handlers do not consume raw source values for basic validation.

#### Scenario: Unknown native option
- **WHEN** a caller provides an option not declared by the selected adapter
- **THEN** navigation reports a strict input diagnostic
- **THEN** dispatch stops before that option reaches an adapter handler

#### Scenario: Declared native option becomes typed handoff
- **WHEN** a caller provides a declared native option value
- **THEN** navigation validates the value through the selected adapter declaration
- **THEN** navigation records source attribution for diagnostics and logging
- **THEN** the selected handler receives the typed native option value or accessor result

### Requirement: Request construction consumes typed resolution results
Navigation MUST construct operation arguments and request envelopes from typed resolution results. Raw argv strings, raw config JSON, and display output are inputs to earlier owners, not request-construction sources. Request construction or dispatch MUST preserve an internal typed selected-adapter native option handoff for the operation without requiring protocol output wrappers or external JSON shapes to change.

#### Scenario: Read request
- **WHEN** typed resolution produces document path, ref, page, and limit
- **THEN** navigation constructs read operation arguments
- **THEN** adapter dispatch receives typed operation input

#### Scenario: Operation includes selected adapter options
- **WHEN** typed resolution produces selected adapter native option values
- **THEN** request construction or dispatch binds those values to the selected adapter input through an internal handoff
- **THEN** the protocol/readable output wrapper remains owned by the output and protocol capabilities
- **THEN** raw config JSON is not forwarded as handler input

### Requirement: Adapter-scoped cost threshold can trigger unstructured full-read outline
Navigation MUST run the unstructured full-read pre-dispatch check before normal adapter outline whenever the effective policy, selected adapter full-read capability declaration, and cost threshold all permit that result. Navigation MUST treat content, cost measurement, and result facts support as parts of the selected adapter's declared full-read capability group.

#### Scenario: Threshold permits full read
- **WHEN** the selected adapter declares full-read capability support
- **AND** navigation determines the full read cost is below the effective threshold
- **THEN** navigation returns the declared unstructured outline result
- **THEN** normal structured outline dispatch is skipped for that request

#### Scenario: Threshold does not permit full read
- **WHEN** the cost threshold is exceeded or support is undeclared
- **THEN** navigation dispatches normal adapter outline

#### Scenario: Full-read capability is partially unsupported
- **WHEN** pre-dispatch policy requires a full-read capability fact that the selected adapter definition does not declare
- **THEN** navigation does not infer support from unrelated adapter methods
- **THEN** navigation uses the documented fallback or reports the unsupported boundary according to the owning policy

### Requirement: Navigation dispatches linked adapter handlers
After successful input resolution and pre-dispatch checks, navigation MUST dispatch to the selected linked adapter handler and return structured result or diagnostic facts to the owning output/protocol layer. Dispatch MUST use the selected adapter definition's operation handler and prepared internal typed native option handoff for the selected operation.

#### Scenario: Dispatch succeeds
- **WHEN** navigation has prepared typed operation input
- **THEN** it calls the selected adapter handler
- **THEN** it preserves the returned structured result facts for projection

#### Scenario: Dispatch uses selected definition handler
- **WHEN** navigation dispatches a selected operation
- **THEN** the operation handler comes from the selected adapter definition or equivalent facade
- **THEN** typed native option values correspond to declarations from the same selected adapter definition
