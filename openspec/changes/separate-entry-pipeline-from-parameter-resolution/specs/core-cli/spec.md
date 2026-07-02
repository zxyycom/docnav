## ADDED Requirements

### Requirement: Core CLI standard entry pipeline MUST preserve command-family boundaries
`docnav` core CLI MUST classify each invocation into an entry family before loading document configuration or resolving document operation parameters. The standard entry pipeline MUST distinguish document operations, config/init/doctor/version, help, and static adapter inspection. Only document operations and explicitly documented document-context inspection paths MAY invoke entry parameter source resolution.

#### Scenario: Help exits before document parameter source resolution
- **WHEN** a caller executes `docnav --help`
- **OR** a caller executes `docnav outline --help`
- **THEN** core classifies the invocation as help
- **THEN** core does not load document operation configuration
- **THEN** core does not invoke entry parameter source resolution
- **THEN** core does not select an adapter or construct a protocol request

#### Scenario: Adapter list keeps static inspection boundary
- **WHEN** a caller executes `docnav adapter list`
- **THEN** core classifies the invocation as static adapter inspection
- **THEN** core does not resolve document operation parameters
- **THEN** output is derived from the core release static registry metadata

#### Scenario: Document command invokes parameter source resolution after classification
- **WHEN** a caller executes `docnav outline docs/guide.md --limit 120`
- **THEN** core classifies the invocation as a document operation
- **THEN** core maps argv into a direct input view
- **THEN** core invokes entry parameter source resolution with direct input, configured sources, explicit adapter native option source descriptors when an adapter owner has declared them, and defaults
- **THEN** adapter routing and protocol request construction consume derived typed runtime values rather than raw argv tokens

### Requirement: Core CLI MUST keep raw argv immutable during parameter resolution
Core CLI parameter source resolution MUST NOT delete, rewrite, reorder, or supplement raw argv tokens. Input-boundary diagnostics, output intent preflight, and document request construction MUST use derived facts or typed runtime values while preserving the original argv as the raw invocation record.

#### Scenario: Config defaults do not mutate argv
- **WHEN** a caller executes `docnav outline docs/guide.md` without `--output`
- **AND** project config supplies `defaults.output`
- **THEN** parameter source resolution may derive an output mode from project config
- **THEN** the raw argv remains unchanged and still lacks `--output`
- **THEN** request construction consumes the derived output value without classifying it as direct input
