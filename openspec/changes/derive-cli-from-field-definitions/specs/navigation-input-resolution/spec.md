## RENAMED Requirements

- FROM: `Core hands raw navigation inputs to navigation`
- TO: `Core hands normalized navigation inputs to navigation`
- FROM: `Navigation selects adapter before adapter parameter extraction`
- TO: `Navigation separates registry projection from selected resolution`

## MODIFIED Requirements

### Requirement: Core hands normalized navigation inputs to navigation

Core MUST hand navigation document operation and routing facts、config source descriptors/paths、the static adapter registry、projected field owner/applicability correspondence and normalized typed/invalid CLI candidates. Core MUST NOT construct selected operation arguments or pass raw native option strings as a parallel semantic input.

#### Scenario: Hand off an outline command

- **WHEN** core successfully parses `docnav outline <path>` and projected fields
- **THEN** navigation receives normalized routing/candidate facts、config descriptors、registry and owner correspondence

### Requirement: Navigation separates registry projection from selected resolution

Navigation MUST expose an operation-scoped registry CLI projection before adapter selection. It MUST derive applicable common declarations and all operation-applicable native declarations from canonical fields plus Docnav extension metadata while preserving field identity and owner/operation correspondence.

Registry projection MAY capture candidates but MUST NOT apply adapter defaults、constraints、handler binding or dispatch semantics. Navigation MUST first resolve routing-required fields and select an adapter, then rebuild a current-operation `FieldDefSet` from common declarations and selected adapter/current-operation declarations. Only identities present in that set MAY enter canonical resolution、materialization、handler binding or dispatch.

#### Scenario: Register multiple adapters without selecting one

- **WHEN** multiple registry adapters expose native fields for an operation
- **THEN** the projection registers all non-conflicting applicable CLI extensions
- **THEN** it preserves the identity and owner needed for later selected-set filtering

#### Scenario: Rebuild after adapter selection

- **WHEN** navigation selects one adapter from the registry
- **THEN** it rebuilds the operation field set from common and selected adapter declarations
- **THEN** candidates whose identities are absent from that set are discarded

#### Scenario: Projection preserves navigation ownership

- **WHEN** core builds help or captures a projected native option
- **THEN** projection supplies command/candidate metadata derived from declarations
- **THEN** navigation still owns adapter selection and selected request construction

### Requirement: Adapter native options are owner-scoped

Navigation MUST validate and resolve native options only when their identities occur in the selected adapter/current-operation `FieldDefSet`. A selected winner or required merge contributor that is invalid MUST produce the canonical field failure. Registry candidates absent from the selected set MUST be discarded before resolution、request construction and dispatch; this boundary MUST NOT create candidate usage state.

#### Scenario: Ignore a candidate outside the selected set

- **WHEN** registry candidates contain a value owned by an unselected adapter
- **THEN** navigation drops it at the selected-set boundary
- **THEN** no request effect or diagnostic is produced from that candidate

#### Scenario: Block a selected invalid option

- **WHEN** an invalid candidate belongs to the selected adapter/current operation and participates in the resolved value
- **THEN** navigation reports the canonical field failure with source facts
- **THEN** dispatch does not occur

### Requirement: Navigation exposes parameter aggregation projections

Navigation MUST derive parameter aggregation from common navigation fields、outline config fields and adapter-id namespaced declarations while preserving each owner. Project-specific field builders MUST attach consumer extension metadata at declaration time; projection functions MUST read canonical and extension metadata from the resulting field sets.

Navigation MUST produce an operation-scoped registry CLI projection、a selected-operation resolution projection and a registry-wide config-inspection projection. CLI projection MUST derive identity、locator、value kind、canonical help/default facts and Docnav presentation extension from those declarations. Duplicate locators within one operation MUST fail deterministically; different operations MAY reuse a locator.

#### Scenario: Build an operation CLI projection

- **WHEN** navigation projects a document operation
- **THEN** it includes applicable common and registry adapter fields with their project extensions
- **THEN** core can augment Clap without reconstructing semantics or presentation

#### Scenario: Use consumer-appropriate projections

- **WHEN** normal execution and `docnav config inspect` read the same valid config object
- **THEN** execution uses only routing and selected-operation fields
- **THEN** inspection uses the registry-wide projection without constructing an effective request

### Requirement: Config source validation uses the selected or inspection projection

After a config source successfully loads as a JSON object, normal document execution MUST extract and validate only routing-required fields and fields declared by the selected current-operation `FieldDefSet`. Unknown、invalid or operation-inapplicable facts outside that projection MUST NOT be read、validated or reported by the invocation.

`docnav config inspect` MUST use the registry-wide config projection and report complete-source unknown key、unknown adapter、shape and typed-value facts without computing source winners、effective request values or dispatch input. Config loading failures remain governed by the origin-aware source-loading requirement.

#### Scenario: Block an invalid selected config value

- **WHEN** selected Markdown consumes `options.docnav-markdown.max_heading_level` and its project value violates the declaration
- **THEN** navigation reports a source-attributed validation failure
- **THEN** dispatch does not occur

#### Scenario: Ignore config outside the selected projection

- **WHEN** a valid config object contains an unknown key、another operation field or any adapter option absent from the selected field set
- **THEN** normal document execution does not inspect or report that fact
- **THEN** it has no request or handler effect

#### Scenario: Inspect the complete source

- **WHEN** `docnav config inspect` reads an unknown adapter id or invalid field outside a document invocation scope
- **THEN** inspection reports the registry-wide source issue
- **THEN** it does not construct or dispatch a document request
