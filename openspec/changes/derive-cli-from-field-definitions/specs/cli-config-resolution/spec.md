## MODIFIED Requirements

### Requirement: Framework Adapter Boundary

Framework-specific extraction MUST remain outside the framework-independent resolution core. The root Cargo workspace MUST provide a Clap companion for CLI strategies and a structured-config companion for config-path strategies; environment extraction MAY remain in core.

The Clap companion MUST own and consume a framework-facing `ClapFieldSpec` projection input. A consumer MAY derive that input from a canonical `FieldDefSet` and consumer-owned extension metadata, but the companion MUST NOT depend on or interpret the consumer's extension payload type. The field declaration MUST remain the authoring source; the runtime input MAY combine canonical identity、locator、value kind、cardinality、accepted/default display and presentation facts. Accepted/default facts in this input are generated-help metadata only.

The companion MUST derive argument registration、capture/decoding behavior and canonical candidate facts only from `ClapFieldSpec`. A present value that cannot be decoded MUST become an invalid `SourceCandidate` retaining field identity、locator、raw input and reason; it MUST NOT abort extraction of unrelated projected fields. Enum、range、pattern、required/default and other canonical semantic constraints MUST NOT be installed as Clap value validation; they remain selected-field resolver responsibilities. Omitted inputs and canonical defaults MUST NOT become explicit CLI candidates.

Clap MUST reject command-shape failures such as unknown flags、duplicate single-value arguments and missing values. Invalid `ClapFieldSpec`、match storage mismatch、source construction failure and declaration conflict MUST remain structural errors. Consumer extension mismatch MUST fail while deriving the project view before the framework bridge. Canonical constraints、source priority、merge、materialization、owner applicability and final diagnostic visibility remain consumer/resolver responsibilities.

Supported CLI projections MUST include string、integer、finite number、Boolean、repeated-string array and repeated `key=value` object. `ValueKind::Json` CLI projection MUST be rejected, and raw strings MUST NOT be interpreted as arbitrary JSON.

#### Scenario: Generate CLI behavior from an extended canonical field

- **WHEN** a consumer declares canonical field facts and attaches its CLI extension through a project builder
- **THEN** the consumer derives one project view and maps it to the companion-owned `ClapFieldSpec`
- **THEN** argument registration、help metadata and candidate identity use that companion input

#### Scenario: Keep the framework bridge outside declarations

- **WHEN** a consumer uses a project-owned projection type that the Clap companion does not know
- **THEN** a higher integration layer maps it to `ClapFieldSpec`
- **THEN** the companion does not depend on the project crate or inspect the project's extension payload

#### Scenario: Preserve field-local decoding failure

- **WHEN** a registered value cannot be decoded for its canonical value kind or Boolean token map
- **THEN** extraction returns an invalid candidate with raw input and reason
- **THEN** unrelated projected fields remain available for later selected-field filtering

#### Scenario: Do not validate canonical semantics during command parsing

- **WHEN** a registered string value is outside its canonical enum or pattern constraint
- **THEN** Clap captures the value without turning the semantic mismatch into a command-shape failure
- **THEN** only a selected field's canonical resolution can report that mismatch

#### Scenario: Keep structural failure separate

- **WHEN** command shape or companion projection/match structure is invalid
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

Docnav MUST declare CLI presentation through project-specific typed-field builder extensions, map the resulting project view to the companion-owned input through one mechanical core bridge, and consume the extraction and resolution APIs intended for library consumers. Its command path MUST preserve documented source priority and materialization while using a single authoritative integration; migrated legacy extraction paths MUST NOT remain as runtime fallbacks.

#### Scenario: Common fields use project builder extensions

- **WHEN** Docnav declares a projected common or routing CLI field
- **THEN** its project builder attaches presentation to the canonical declaration
- **THEN** core maps the derived project view to the companion-owned input without re-authoring field facts

#### Scenario: Remove compatibility paths

- **WHEN** extension projection、extraction and selected resolution equivalence tests pass
- **THEN** the authoritative integration replaces the migrated legacy extraction paths
- **THEN** rollback requires reverting the change
