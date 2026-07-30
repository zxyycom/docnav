Current JSON readable 行为仍是 generic `readable-view`；本临时 delta spec 只固定 Target JSON 专用 presentation 已确认的 output/raw/ref 边界。逐 operation presentation 与 selection semantics 尚未决定，因此在 design 开放问题关闭并补全 requirements 前不得实施。

## ADDED Requirements

### Requirement: JSON format-specific presentation remains inside readable-view
Docnav MUST deliver any approved JSON format-specific presentation as a presentation strategy within the existing `readable-view` output path. The presentation MUST be owned by the output layer and MUST consume the same immutable `ProtocolResponse` produced by navigation. It MUST NOT create another public output mode、serialized renderer identity or adapter-owned presentation contract. The exact JSON presentation and linked selection mechanics MUST be specified by this change before implementation.

#### Scenario: Approved JSON presentation uses the existing output path
- **WHEN** a JSON document response is rendered with the approved format-specific presentation
- **THEN** the invocation still uses public output mode `readable-view`
- **THEN** the selected presentation receives the navigation-produced `ProtocolResponse` unchanged
- **THEN** no renderer identity is added to CLI、config or serialized output

### Requirement: JSON readable presentation uses only existing raw facts
The JSON format-specific presentation MUST derive every readable fact from values already present in the supplied `ProtocolResponse`. It MUST NOT invoke the JSON adapter、re-read the document、obtain adapter-private state or write presentation-only facts back into an adapter result. If an approved presentation requires a fact that the response does not contain, this change MUST remain blocked or explicitly change the owning contract in a separately approved scope; the renderer MUST NOT synthesize that fact.

#### Scenario: Existing response facts are sufficient
- **WHEN** an approved JSON readable projection is produced
- **THEN** every displayed value is traceable to an existing response fact or an explicitly approved presentation-only derivation of those facts
- **THEN** rendering performs no adapter invocation or document read

#### Scenario: Desired fact is absent
- **WHEN** a proposed JSON presentation needs a fact absent from the `ProtocolResponse`
- **THEN** the renderer does not infer that fact from adapter-private knowledge
- **THEN** implementation remains blocked until the presentation is revised or the owning contract change is separately approved

### Requirement: JSON readable presentation preserves opaque ref boundaries
Any JSON ref carried from a response into readable output MUST remain the complete opaque string supplied by the adapter. The output layer MUST NOT parse ref tokens or use ref spelling to synthesize hierarchy、depth、parentage or indentation. The approved presentation contract MUST separately state whether and how the complete ref is used as a path-location signal.

#### Scenario: Ref remains opaque
- **WHEN** an approved JSON readable projection includes a ref
- **THEN** the output preserves the complete ref string
- **THEN** no output fact is derived by decoding the ref grammar
- **THEN** no hierarchy、depth、parent or indentation fact is synthesized from the ref

### Requirement: JSON presentation does not change the machine contract
Adding JSON format-specific readable presentation MUST NOT change `ProtocolResponse`、`protocol-json` serialization、protocol schema or examples、JSON adapter result facts、ref spelling、result ordering、cost、page、diagnostic mapping or the accepted public output values. Other formats MUST retain their Current renderer behavior unless a separately approved contract changes them.

#### Scenario: Raw and readable consume one response
- **WHEN** the same JSON operation is observed through `protocol-json` and the approved JSON `readable-view`
- **THEN** `protocol-json` serializes the unchanged response under the existing schema
- **THEN** readable output uses only that response without changing its raw facts

#### Scenario: Other formats are unaffected
- **WHEN** a document format has no separately approved format-specific presentation
- **THEN** its Current renderer behavior remains unchanged
- **THEN** the JSON change does not add a new public selection value or otherwise change that format's contract

### Requirement: JSON presentation delivery has contract and parity evidence
Before JSON format-specific presentation is reported as delivered, the output owner MUST define exact observable requirements for every supported operation and branch chosen by this change. Repository evidence MUST verify those requirements at the output contract、real CLI and canonical package boundaries, and MUST compare readable facts with the corresponding schema-valid `protocol-json` response. Expected readable text MUST be independent of the renderer implementation, and readable output MUST remain a text contract rather than a second machine schema.

#### Scenario: Approved contract is verifiable
- **WHEN** all design open questions have been resolved
- **THEN** this delta contains exact scenarios for each supported operation、branch、presentation rule and selection rule
- **THEN** tests can distinguish the approved output from a different presentation without using renderer helpers to generate expectations

#### Scenario: Delivery evidence preserves raw parity
- **WHEN** repository and canonical package validation exercise the approved JSON readable presentation
- **THEN** corresponding `protocol-json` responses validate against the existing protocol schema
- **THEN** readable assertions trace to the same refs、ordering、content、cost、page and other applicable raw facts
