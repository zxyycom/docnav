## MODIFIED Requirements

### Requirement: Core CLI strictly handles public argv

Core CLI MUST use one authoritative Clap tree for root commands、subcommands、fixed positionals、core-owned static arguments、help、the existing `version` command and registry-projected document fields. It MUST reject flags absent from the static and registry-projected shape、duplicate single-value input and missing values according to the strict argv policy.

Registry-projected adapter flags are recognized before adapter selection. Candidate extraction MUST use canonical field facts and Docnav extension metadata derived from the declaration. Navigation's selected `FieldDefSet` determines whether a captured field participates in the request.

After successful structural parsing, core MUST obtain normalized typed/invalid candidates through the Clap companion, validate explicit `output` and hand routing facts、candidates and owner/applicability correspondence to navigation. Core MUST NOT maintain parallel accepted-value/default/presentation tables、derive projected Clap ids independently、scan argv again for business fields or apply adapter-owned constraints.

#### Scenario: Reject an unknown flag

- **WHEN** a caller passes a flag absent from the command's static and registry-projected shape
- **THEN** authoritative Clap parsing returns a caller-input failure
- **THEN** the flag is not forwarded

#### Scenario: Generate a common argument

- **WHEN** an operation projection contains `adapter`、`page`、`limit`、`output` or `pagination.enabled`
- **THEN** core registers and extracts it through canonical metadata and the project extension projection
- **THEN** core does not duplicate semantic or presentation metadata

#### Scenario: Reject static/projection conflict

- **WHEN** a projected locator or argument id conflicts with a static argument、help or version surface
- **THEN** augmentation returns a deterministic declaration conflict before parsing

#### Scenario: Forward a registry candidate before selection

- **WHEN** authoritative parsing captures an operation-applicable adapter flag that is absent from the later selected field set
- **THEN** core hands the normalized candidate to navigation without an early applicability decision
- **THEN** navigation's selected-set boundary determines that it has no request effect

#### Scenario: Validate a selected common field canonically

- **WHEN** a current-operation `page` or `limit` candidate fails decoding or canonical range validation
- **THEN** the selected field resolution reports the canonical caller-input failure before dispatch
- **THEN** no parallel argv decoder supplies the result

#### Scenario: Preserve a hyphen-leading value

- **WHEN** a string or path option value begins with `-`
- **THEN** `--flag=<value>` preserves it unambiguously
- **THEN** a separated flag-shaped token follows normal Clap parsing

#### Scenario: Help and version avoid navigation side effects

- **WHEN** a caller requests help or invokes `docnav version`
- **THEN** core returns a PlainText outcome
- **THEN** it does not load document config、select an adapter or dispatch an operation

### Requirement: Core CLI selects output mode and process exit behavior

Core CLI MUST parse requested output mode、call the output contract for document projection and map diagnostics to process exit behavior without redefining protocol or readable payload semantics.

Failure rendering MUST use an explicit output candidate only after successful extraction and canonical validation, or a later config/default output selected by normal navigation resolution. Command-shape failure、duplicate output、invalid output and failures before a valid output is known MUST use PlainText. Core MUST NOT infer output mode from raw malformed argv.

#### Scenario: Early parse failure is PlainText

- **WHEN** authoritative parsing fails for an unknown flag、duplicate single-value argument or missing value
- **THEN** core emits the normal PlainText caller-input failure and exit behavior

#### Scenario: Invalid output is PlainText

- **WHEN** explicit `output` fails decoding or canonical validation
- **THEN** core reports the failure through PlainText
- **THEN** the invalid value is not used as renderer selection

#### Scenario: Valid output selects later failure rendering

- **WHEN** explicit `output` is valid and a later selected-field resolution or adapter step fails
- **THEN** core uses that document output mode

#### Scenario: Omitted output follows normal resolution

- **WHEN** explicit `output` is absent
- **THEN** config/default output may be selected only through normal navigation resolution

#### Scenario: Protocol JSON has no readable framing

- **WHEN** a valid invocation selects `protocol-json`
- **THEN** core emits protocol stdout without readable-view framing

#### Scenario: Failure exit follows CLI mapping

- **WHEN** a document invocation fails
- **THEN** core uses the exit behavior for the surfaced diagnostic class
- **THEN** diagnostic/output contracts retain failure payload ownership
