本 delta spec 修改 navigation input resolution，使 navigation 从 selected adapter definition 消费 adapter-owned declarations，并通过内部 dispatch boundary 向 handler 交付 typed native option values/accessors。实施和归档前，主规范与当前二进制状态仍以 `docs/`、代码和测试为准。

## MODIFIED Requirements

### Requirement: Navigation selects adapter before adapter parameter extraction
Navigation MUST select the adapter using routing inputs and registry facts before extracting selected adapter native options. The selected registry entry MUST expose an adapter definition, and navigation MUST consume native option declarations and capability declarations from that selected definition.

#### Scenario: Multiple adapters exist
- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only the selected adapter's native declarations are used for extraction
- **THEN** declarations from unselected adapter definitions remain outside the operation field set

#### Scenario: Selected definition provides capability facts
- **WHEN** navigation has selected an adapter
- **THEN** navigation reads optional capability declarations from the selected adapter definition
- **THEN** pre-dispatch policy uses only those declared support facts

#### Scenario: Navigation receives selected definition as the fact source
- **WHEN** core registry returns a selected adapter entry
- **THEN** the selected entry provides the adapter definition used for declaration registration, full-read pre-dispatch, and dispatch
- **THEN** navigation uses definition-provided adapter-owned native option and capability semantics

### Requirement: Selected adapter declarations own parameter facts
Selected adapter typed-field declarations MUST provide adapter-owned option identity, extraction metadata, defaults, validation facts, operation applicability, and internal typed handoff/accessor binding metadata used during navigation resolution. These declarations MUST come from the selected adapter definition. The same declaration MUST drive extraction and handler binding for request construction and dispatch.

#### Scenario: Selected Markdown adapter
- **WHEN** Markdown is selected
- **THEN** Markdown native option declarations are registered for extraction from the selected adapter definition
- **THEN** non-Markdown option declarations remain outside the selected declaration set
- **THEN** resolved Markdown option values can be handed to the Markdown handler as typed native option values through the internal dispatch boundary

#### Scenario: Selected adapter declaration binds typed handoff
- **WHEN** a selected adapter declaration includes typed handoff or accessor binding metadata
- **THEN** navigation uses that metadata after validation and extraction
- **THEN** request construction or dispatch prepares the adapter-specific typed option value for the handler

#### Scenario: Selected declaration binds dispatch
- **WHEN** navigation has validated a selected adapter native option
- **THEN** the handler binding comes from the same selected declaration used for extraction
- **THEN** request construction uses that declaration as the adapter-owned mapping for that option

### Requirement: Adapter native options are owner-scoped
Navigation MUST validate and extract native options only when they are declared by the selected adapter definition. Undeclared owner-scoped options MUST fail strictly. Declared native options MUST be resolved into typed values before dispatch, and handlers receive those typed values or accessors for native-option consumption.

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
Navigation MUST construct operation arguments, request envelopes, and handler-facing adapter input from typed resolution results. Raw argv strings, raw config JSON, and display output are inputs to earlier owners. Request construction or dispatch MUST preserve an internal typed selected-adapter native option handoff/accessor for the operation while protocol output wrappers and external JSON shapes remain under their existing owners.

#### Scenario: Read request
- **WHEN** typed resolution produces document path, ref, page, and limit
- **THEN** navigation constructs read operation arguments
- **THEN** adapter dispatch receives typed operation input

#### Scenario: Operation includes selected adapter options
- **WHEN** typed resolution produces selected adapter native option values
- **THEN** request construction or dispatch binds those values to the selected adapter input through an internal handoff
- **THEN** the protocol/readable output wrapper remains owned by the output and protocol capabilities
- **THEN** raw config JSON is not forwarded as handler input

#### Scenario: Protocol-stable options remain separate from handler input
- **WHEN** protocol request construction includes `OperationArguments.options`
- **THEN** navigation constructs that protocol-stable object from typed resolution results
- **THEN** handler-facing typed native option handoff/accessor remains the dispatch contract for declared typed bindings
- **THEN** protocol output wrapper shape remains separate from adapter handler input typing

### Requirement: Adapter-scoped cost threshold can trigger unstructured full-read outline
Navigation MUST run the unstructured full-read pre-dispatch check before normal adapter outline whenever the effective policy, selected adapter full-read capability declaration, and cost threshold all permit that result. Navigation MUST treat support, content, cost measurement, and result facts as parts of the selected adapter's declared full-read capability group.

#### Scenario: Threshold permits full read
- **WHEN** the selected adapter declares full-read capability support
- **AND** navigation determines the full read cost is below the effective threshold
- **THEN** navigation returns the declared unstructured outline result
- **THEN** normal structured outline dispatch is skipped for that request

#### Scenario: Threshold selects normal outline
- **WHEN** the cost threshold is exceeded or support is undeclared
- **THEN** navigation dispatches normal adapter outline

#### Scenario: Full-read capability is partially unsupported
- **WHEN** pre-dispatch policy requires a full-read capability fact outside the selected adapter definition
- **THEN** navigation follows the documented fallback or reports the unsupported boundary according to the owning policy
- **THEN** navigation bases that decision on the selected adapter definition's full-read capability group

### Requirement: Navigation dispatches linked adapter handlers
After successful input resolution and pre-dispatch checks, navigation MUST dispatch to the selected linked adapter handler and return structured result or diagnostic facts to the owning output/protocol layer. Dispatch MUST use the selected adapter definition's operation handler and prepared internal typed native option handoff/accessor for the selected operation.

#### Scenario: Dispatch succeeds
- **WHEN** navigation has prepared typed operation input
- **THEN** it calls the selected adapter handler
- **THEN** it preserves the returned structured result facts for projection

#### Scenario: Dispatch uses selected definition handler
- **WHEN** navigation dispatches a selected operation
- **THEN** the operation handler comes from the selected adapter definition
- **THEN** typed native option values correspond to declarations from the same selected adapter definition

#### Scenario: Dispatch preserves single-definition ownership
- **WHEN** navigation calls a selected operation handler
- **THEN** the handler handle, capability context, and native option handoff all originate from the selected adapter definition
- **THEN** navigation dispatch uses a coherent selected definition for handler handles and native option declarations
