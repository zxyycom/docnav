本 delta 的核心目标是让 `cli-config-resolution-clap` 完整拥有 canonical CLI projection 和 typed extraction；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 spec，不影响现有主规范或其它文档。

## MODIFIED Requirements

### Requirement: Framework Adapter Boundary
Framework-specific extraction MUST remain outside the framework-independent resolution core. The Cargo workspace MUST provide a Clap companion for CLI strategies and a structured-config companion for config-path strategies; environment extraction MAY remain in the core because it requires no external framework.

The Clap companion MUST own flag validation, argument id derivation, short/long registration, action/cardinality, argument conflict detection, typed value parser selection, typed `ArgMatches` reads, and conversion to canonical CLI `SourceCandidate` values. It MUST support string, integer, finite-number, `SetTrue` boolean, repeated-string array, and repeated `key=value` object projections. Clap value parsers MUST reject lexical type or `key=value` shape failures before a successful `Source` is returned.

The companion MUST reject a CLI `ValueKind::Json` projection with `ClapProjectionError::UnsupportedValueKind`. It MUST NOT guess arbitrary JSON from raw text, and consumers MUST NOT duplicate projection mechanics or post-parse dynamic strings.

#### Scenario: Use Clap companion with canonical parameters
- **WHEN** a consumer passes a canonical `FieldDefSet` containing supported CLI strategies
- **THEN** the companion registers value-kind-appropriate Clap arguments and typed parsers
- **THEN** it returns candidates for canonical identities without a parallel field model or consumer decoder

#### Scenario: Clap stores typed scalar values
- **WHEN** canonical CLI fields declare string, integer, finite-number, or `SetTrue` boolean kinds
- **THEN** Clap stores the corresponding typed values before extraction
- **THEN** extraction reads them without string parsing

#### Scenario: String that resembles JSON remains a string
- **WHEN** a canonical string field receives CLI text `true`
- **THEN** the string parser stores `true` as a string
- **THEN** extraction does not convert it to boolean or JSON

#### Scenario: Repeated collection values use typed entries
- **WHEN** canonical array and object fields receive repeated CLI occurrences
- **THEN** Clap stores repeated strings for the array and validated `key=value` entries for the object
- **THEN** extraction mechanically constructs the canonical collection value

#### Scenario: Invalid lexical value fails at Clap boundary
- **WHEN** an integer, finite-number, or object entry cannot be decoded by its registered parser
- **THEN** authoritative Clap parsing returns a structured value error
- **THEN** no successful CLI `Source` contains a guessed or post-parsed value

#### Scenario: Reject arbitrary JSON CLI decoding
- **WHEN** a canonical field combines a CLI locator with `ValueKind::Json`
- **THEN** the companion returns `ClapProjectionError::UnsupportedValueKind`
- **THEN** it does not interpret the raw string as arbitrary JSON

#### Scenario: Use structured config companion with canonical parameters
- **WHEN** a consumer passes structured config and a canonical set containing config-path strategies
- **THEN** the config companion returns candidates for declared identities
- **THEN** resolution receives them through the same source model used by CLI and environment extraction

#### Scenario: Extract environment variables without ambient global state
- **WHEN** a consumer passes iterable environment key/value pairs and a canonical parameter set
- **THEN** the extractor reads only declared environment locators
- **THEN** consumers and tests do not require process-global environment access

### Requirement: Docnav Hard Cutover Boundary
Docnav MUST consume the canonical parameter set through the same extraction and resolution APIs intended for external consumers. It MUST preserve Docnav-owned CLI, config, adapter, protocol, diagnostic, and output behavior without a runtime fallback or separate dynamic CLI decoder.

#### Scenario: Remove parallel field-set conversion
- **WHEN** Docnav resolves navigation fields and selected adapter native options
- **THEN** canonical typed-fields metadata goes directly to extraction and resolution
- **THEN** the runtime path does not copy field types or constraints into another model

#### Scenario: Preserve Docnav source priority
- **WHEN** Docnav supplies explicit, project, user, and built-in sources
- **THEN** resolution preserves the documented priority
- **THEN** Docnav retains adapter applicability, request construction, diagnostic mapping, and output ownership

#### Scenario: Map parsed static input through a consumer-owned source adapter
- **WHEN** Docnav holds parsed core-owned static CLI input before resolution
- **THEN** a Docnav-private adapter MAY traverse canonical processing metadata to emit `SourceCandidate` values
- **THEN** it does not maintain parallel locator, type, constraint, default, validation, or merge metadata

#### Scenario: Dynamic native input bypasses string bridges
- **WHEN** Docnav parses adapter native CLI input
- **THEN** `cli-config-resolution-clap` produces the canonical typed CLI source
- **THEN** Docnav does not construct `flag + String`, guess a Clap id, or attempt JSON decoding

#### Scenario: Complete cutover removes compatibility paths
- **WHEN** the canonical Clap/resolution path passes equivalence tests
- **THEN** business argv scanners, dynamic string decoders, old resolver paths, runtime switches, and field-model compatibility wrappers are absent
- **THEN** rollback requires reverting the code change rather than enabling a fallback
