## MODIFIED Requirements

### Requirement: Framework Adapter Boundary

Framework-specific extraction MUST remain outside the framework-independent resolution core. The Cargo workspace MUST provide a Clap companion for CLI strategies and a structured-config companion for config-path strategies; environment extraction MAY remain in core.

The Clap companion MUST consume a projection derived from a canonical `FieldDefSet` and consumer-owned extension metadata. The field declaration MUST remain the authoring source; a runtime projection view MAY combine canonical identity、locator、value kind、constraints/default display and presentation.

The companion MUST derive argument identity、locator、cardinality、capture/decoding behavior and canonical candidate facts from that projection. A present value that cannot be decoded MUST become an invalid `SourceCandidate` retaining field identity、locator、raw input and reason; it MUST NOT abort extraction of unrelated projected fields. Omitted inputs and canonical defaults MUST NOT become explicit CLI candidates.

Clap MUST reject command-shape failures such as unknown flags、duplicate single-value arguments and missing values. Projection/extension mismatch、match storage mismatch、source construction failure and declaration conflict MUST remain structural errors. Canonical constraints、source priority、merge、materialization、owner applicability and final diagnostic visibility remain consumer/resolver responsibilities.

Supported CLI projections MUST include string、integer、finite number、Boolean、repeated-string array and repeated `key=value` object. `ValueKind::Json` CLI projection MUST be rejected, and raw strings MUST NOT be interpreted as arbitrary JSON.

#### Scenario: Generate CLI behavior from an extended canonical field

- **WHEN** a consumer declares canonical field facts and attaches its CLI extension through a project builder
- **THEN** the consumer extractor derives one companion projection from that field
- **THEN** argument registration、help metadata and candidate identity use that projection

#### Scenario: Preserve field-local decoding failure

- **WHEN** a registered value cannot be decoded for its canonical value kind or Boolean token map
- **THEN** extraction returns an invalid candidate with raw input and reason
- **THEN** unrelated projected fields remain available for later selected-field filtering

#### Scenario: Keep structural failure separate

- **WHEN** command shape or projection/match/extension structure is invalid
- **THEN** Clap or the companion returns the corresponding structural error
- **THEN** the failure is not represented as a caller field candidate

#### Scenario: String remains string

- **WHEN** a canonical string field receives CLI text `true`
- **THEN** extraction preserves it as a string candidate
- **THEN** the companion does not reinterpret it as Boolean or JSON

#### Scenario: Canonical default remains fallback

- **WHEN** a field has a static default and its CLI flag is omitted
- **THEN** extraction records no explicit CLI candidate
- **THEN** resolver fallback supplies the default

#### Scenario: Structured config remains independent

- **WHEN** a consumer passes structured config and canonical config-path strategies
- **THEN** the config companion returns candidates for declared identities
- **THEN** it does not depend on the Clap companion or CLI extensions

#### Scenario: Environment extraction remains injectable

- **WHEN** a consumer passes iterable environment key/value pairs and canonical environment strategies
- **THEN** extraction reads declared locators without process-global environment ownership

### Requirement: Docnav Hard Cutover Boundary

Docnav MUST declare CLI presentation through project-specific typed-field builder extensions and consume the resulting projection through the extraction and resolution APIs intended for library consumers. Its command path MUST preserve documented source priority and materialization while using a single authoritative integration; migrated legacy extraction paths MUST NOT remain as runtime fallbacks.

#### Scenario: Common fields use project builder extensions

- **WHEN** Docnav declares a projected common or routing CLI field
- **THEN** its project builder attaches presentation to the canonical declaration
- **THEN** the companion receives a projection derived from that declaration

#### Scenario: Remove compatibility paths

- **WHEN** extension projection、extraction and selected resolution equivalence tests pass
- **THEN** the authoritative integration replaces the migrated legacy extraction paths
- **THEN** rollback requires reverting the change
