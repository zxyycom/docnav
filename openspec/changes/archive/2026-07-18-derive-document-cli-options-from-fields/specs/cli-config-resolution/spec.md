## MODIFIED Requirements

### Requirement: Framework Adapter Boundary

Framework-specific extraction MUST remain outside the framework-independent resolution core. The root Cargo workspace MUST provide a Clap companion for CLI strategies and a structured-config companion for config-path strategies; environment extraction MAY remain in the core because it requires no external framework.

The Clap companion MUST directly consume a canonical `FieldDefSet` and its CLI processing projection. It MUST derive argument identity、flag registration、value capture、help/value name、canonical accepted/default display and candidate field identity from that projection. It MUST use canonical field facts for value kind、constraints、default and merge semantics. It MUST support string、integer、finite-number、valueless Boolean switch、declared token-to-Boolean mapping、repeated-string array and repeated `key=value` object inputs. It MUST reject `ValueKind::Json` CLI projections with `ClapProjectionError::UnsupportedValueKind` and MUST NOT decode a raw CLI string as arbitrary JSON.

Clap MUST own structural command-shape failures such as unknown arguments、duplicate single-value inputs and missing values. After structural parsing succeeds, a value-kind decoding failure MUST become an invalid `SourceCandidate` that retains canonical field identity、locator、raw input and reason while unrelated projected candidates remain available. Enum、range、pattern、required/default and merge semantics MUST remain canonical resolution concerns rather than eager Clap value validation. An omitted input MUST NOT become an explicit candidate, and a static default MUST remain resolver fallback.

#### Scenario: Use Clap companion with canonical parameters

- **WHEN** a caller passes a canonical `FieldDefSet` containing supported CLI strategies to `cli-config-resolution-clap`
- **THEN** the companion registers the declared arguments and returns candidates for canonical field identities
- **THEN** registration、help and extraction consume one field projection

#### Scenario: Generate help from authored and canonical facts

- **WHEN** a projected field declares help/value name and its canonical field declares accepted values or a static default
- **THEN** the Clap argument combines those facts in generated help
- **THEN** accepted/default facts are not repeated in CLI-only metadata

#### Scenario: Preserve a decode failure as candidate data

- **WHEN** structural parsing captures a registered value that cannot be decoded for its canonical value kind or declared Boolean token mapping
- **THEN** extraction returns an invalid candidate with field identity、locator、raw input and reason
- **THEN** extraction does not turn the field-local mismatch into an unrelated command-shape failure

#### Scenario: Reject arbitrary JSON CLI decoding

- **WHEN** a canonical field combines a CLI flag processing locator with `ValueKind::Json`
- **THEN** `cli-config-resolution-clap` rejects the projection with `ClapProjectionError::UnsupportedValueKind`
- **THEN** it does not interpret the raw CLI string as an arbitrary JSON candidate

#### Scenario: Use structured config companion with canonical parameters

- **WHEN** a caller passes a structured config document and a canonical `FieldDefSet` containing config-path strategies to the config companion
- **THEN** the companion returns candidates for declared canonical field identities
- **THEN** the resolution core receives them through the same candidate/source model used by CLI and env extraction

#### Scenario: Extract environment variables without ambient global state

- **WHEN** a caller passes an iterable collection of environment key/value pairs and a canonical parameter set to the env extractor
- **THEN** the extractor reads only declared environment locators
- **THEN** tests and callers can supply input without requiring the extractor to own process-global environment access

### Requirement: Docnav Hard Cutover Boundary

Docnav MUST consume the root-workspace canonical parameter set directly through the extraction and resolution APIs. The field-derived path MUST preserve the owner-documented Docnav CLI、config、adapter、protocol、diagnostic and output behavior and MUST be the only runtime document option path after cutover.

#### Scenario: Use one canonical declaration path

- **WHEN** Docnav registers or resolves common document fields and adapter native options
- **THEN** it passes canonical typed-field metadata directly to the Clap companion and resolution
- **THEN** one canonical declaration supplies identity、type、constraint、default and CLI projection facts

#### Scenario: Hand off normalized CLI candidates

- **WHEN** document argv passes structural parsing
- **THEN** Docnav hands normalized typed/invalid `SourceCandidate` values to navigation using canonical field identities
- **THEN** navigation can determine selected applicability from candidate identity and selected field membership

#### Scenario: Preserve Docnav source priority

- **WHEN** Docnav maps explicit input、project config、user config and built-in defaults into ordered sources
- **THEN** resolution matches Docnav's documented priority behavior
- **THEN** Docnav remains the owner of adapter applicability、request construction、diagnostic-code mapping and output projection

#### Scenario: Complete cutover leaves one runtime path

- **WHEN** field projection、candidate extraction and public behavior equivalence tests pass
- **THEN** registration、candidate extraction and selected resolution use the field-derived path
- **THEN** the previous catalog、raw remapping、parallel decoder and runtime fallback are absent from the document command path
