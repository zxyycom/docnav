## ADDED Requirements

### Requirement: Core-owned config processing is stage-scoped

Core-owned document stages MUST select an explicit config projection and MUST surface only facts required by that projection. Runtime invocation logging initialization MUST read and validate only its documented logging field paths and the structural ancestors required to reach them; that projection MUST NOT treat the remaining config object as its schema. Facts from other config sections、operations、adapter namespaces、fields or sibling keys MUST NOT affect the logging-stage outcome, even when a shared parser or validator computes them internally.

Core MUST preserve separate full-source behavior for `docnav config inspect`. Help and `version` MUST remain outside document config processing.

#### Scenario: Logging stage ignores unrelated config facts

- **WHEN** invocation logging initialization reads a valid logging projection from a JSON object that also contains an invalid unrelated document field
- **THEN** the logging stage uses only its projected fields
- **THEN** the unrelated fact produces no logging-stage diagnostic or side effect

#### Scenario: Logging field failure remains visible

- **WHEN** a field required by the logging projection has an invalid value
- **THEN** the logging stage reports its existing core-owned config diagnostic
- **THEN** it does not report sibling issues outside the logging projection

## MODIFIED Requirements

### Requirement: Core CLI strictly handles public argv

Core CLI MUST use one authoritative Clap tree for root commands、subcommands、fixed positionals、core-owned static arguments、help、the existing `version` command and registry-projected document fields. It MUST reject flags absent from the static and registry-projected shape、duplicate single-value input and missing values according to the strict argv policy.

Registry-projected adapter flags are recognized before adapter selection. Candidate extraction MUST use the companion-owned `ClapFieldSpec` produced by core's mechanical conversion from canonical field facts and Docnav extension metadata. Navigation's selected `FieldDefSet` determines whether a successfully captured typed/invalid field candidate participates in the request.

After successful structural parsing, core MUST obtain normalized typed/invalid candidates through the Clap companion, validate explicit `output` and hand routing facts、candidates and owner/applicability correspondence to navigation. Core MUST NOT maintain parallel accepted-value/default/presentation tables、derive projected Clap ids independently、scan argv again for business fields or apply adapter-owned constraints.

Command-shape failures are not field candidates. Unknown flags、duplicate single-value input、missing values and token-boundary failures MUST block before adapter selection even when the affected registered flag would later belong to an unselected adapter.

#### Scenario: Reject an unknown flag

- **WHEN** a caller passes a flag absent from the command's static and registry-projected shape
- **THEN** authoritative Clap parsing returns a caller-input failure
- **THEN** the flag is not forwarded

#### Scenario: Generate a common argument

- **WHEN** an operation projection contains `adapter`、`page`、`limit`、`output` or `pagination.enabled`
- **THEN** core registers and extracts it through canonical metadata and the project extension projection
- **THEN** core does not duplicate semantic or presentation metadata

#### Scenario: Bridge a project projection mechanically

- **WHEN** core receives a `DocnavFieldProjection` for a generated field
- **THEN** it copies the derived facts into the companion-owned `ClapFieldSpec`
- **THEN** it does not recompute accepted values、defaults、owner or operation applicability

#### Scenario: Reject static/projection conflict

- **WHEN** a projected locator or argument id conflicts with a static argument、help or version surface
- **THEN** augmentation returns a deterministic declaration conflict before parsing

#### Scenario: Forward a registry candidate before selection

- **WHEN** authoritative parsing captures an operation-applicable adapter flag that is absent from the later selected field set
- **THEN** core hands the normalized candidate to navigation without an early applicability decision
- **THEN** navigation's selected-set boundary determines that it has no request effect

#### Scenario: Structural failure precedes selected filtering

- **WHEN** a caller repeats a registered single-value adapter flag or omits its required value
- **THEN** authoritative Clap parsing returns a command-shape failure before adapter selection
- **THEN** the failure is not converted into a discardable field candidate

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

Failure rendering MUST use an explicit output candidate only after successful extraction and canonical validation, or a later config/default output selected by successful normal navigation resolution. Command-shape failure、duplicate output、invalid output and failures before a valid output is known MUST use PlainText. A valid explicit output MUST control failures after structural parsing; when explicit output is absent, config/default output MUST NOT control failures that occur before normal navigation resolution produces a valid output value. Core MUST NOT infer output mode from raw malformed argv.

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
- **THEN** config/default output may be selected only through successful normal navigation resolution
- **THEN** an earlier routing、selection or field-resolution failure uses PlainText

#### Scenario: Resolved config output controls adapter failure

- **WHEN** explicit output is absent、normal navigation resolution selects config/default `protocol-json` and a later adapter operation fails
- **THEN** core projects that failure through protocol JSON

#### Scenario: Protocol JSON has no readable framing

- **WHEN** a valid invocation selects `protocol-json`
- **THEN** core emits protocol stdout without readable-view framing

#### Scenario: Failure exit follows CLI mapping

- **WHEN** a document invocation fails
- **THEN** core uses the exit behavior for the surfaced diagnostic class
- **THEN** diagnostic/output contracts retain failure payload ownership
