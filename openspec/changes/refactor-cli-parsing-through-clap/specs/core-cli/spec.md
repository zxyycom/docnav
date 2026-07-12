本 delta 的核心目标是让 core 使用一棵 authoritative Clap command tree，并只从 typed matches 构造 CLI command facts；本文是仅位于 `openspec/changes/refactor-cli-parsing-through-clap/` 的未审核临时 spec，不影响现有主规范或其它文档。

## MODIFIED Requirements

### Requirement: Core CLI strictly handles public argv
Core CLI MUST use one authoritative Clap command tree for root commands, subcommands, fixed positionals, core-owned arguments, help/version, and registry-declared adapter native arguments. It MUST reject unknown, duplicate, unsupported, unused, missing, or lexically invalid public input according to the strict argv policy.

Core-owned value arguments MUST register their lexical, enum, or range parser with Clap. Dynamic adapter arguments MUST be registered and extracted only through `cli-config-resolution-clap`. Command construction MUST read typed matches and MUST NOT run a second business token scanner, rebuild native arguments as string-valued long flags, derive dynamic Clap ids independently, or decode native strings after parsing.

#### Scenario: Unknown flag
- **WHEN** a caller passes an unknown flag to a document command
- **THEN** the authoritative Clap parse produces an invalid request diagnostic through core mapping
- **THEN** the flag is not forwarded to navigation or adapters

#### Scenario: Operation-inapplicable known flag
- **WHEN** a caller passes a known flag to an operation whose command shape does not declare it
- **THEN** the authoritative Clap parse rejects the flag as strict caller input
- **THEN** core does not reclassify it with another business token scanner

#### Scenario: Core value is decoded by Clap
- **WHEN** a caller provides output, pagination, page, limit, or another core-owned value argument
- **THEN** its registered Clap value parser returns a typed match or a Clap parse failure
- **THEN** command-model construction does not invoke another string decoder

#### Scenario: Dynamic native option is projected by the companion
- **WHEN** an operation-scoped registry projection declares an adapter native CLI flag
- **THEN** core uses `cli-config-resolution-clap` to register and extract that argument
- **THEN** a successful parse hands navigation a typed CLI source candidate rather than a raw string

#### Scenario: Hyphen-leading value uses inline syntax
- **WHEN** a caller needs to pass a string or path value beginning with `-`
- **THEN** `--flag=<value>` preserves that value unambiguously
- **THEN** a separated token such as `--query --future` remains flag-shaped input

#### Scenario: Help is a display outcome
- **WHEN** a caller requests root or subcommand help
- **THEN** core returns the Clap help display outcome through the PlainText success channel
- **THEN** core does not resolve project context, load config, select an adapter, or dispatch a document operation

### Requirement: Core CLI selects output mode and process exit behavior
Core CLI MUST parse requested output mode, call the output contract for projection, and map diagnostics to process exit behavior without redefining protocol or readable payload semantics.

Before authoritative parsing, one bounded presentation probe MUST recover only the document operation and syntactically valid output occurrences from raw argv. The last valid occurrence MUST be the failure-presentation hint; if none exists, the hint MUST be `readable-view`. Authoritative Clap parsing remains responsible for accepting or rejecting the invocation.

The probe MUST NOT determine parse success, construct a command, load config, select an adapter, validate navigation/native fields, classify unknown or unused arguments, or expose any other field fact.

#### Scenario: Protocol-json requested
- **WHEN** a caller requests `--output protocol-json`
- **THEN** core asks the protocol/output pipeline for protocol stdout
- **THEN** readable-view framing is not emitted to stdout

#### Scenario: Strict parse failure preserves requested protocol output
- **WHEN** a document command contains a valid `--output protocol-json` and another argument causes authoritative parsing to fail
- **THEN** the presentation probe identifies the output context without decoding business input
- **THEN** core emits the existing protocol failure projection and exit behavior

#### Scenario: Duplicate output remains a strict parse failure
- **WHEN** argv contains more than one output occurrence
- **THEN** the presentation probe retains only the last syntactically valid mode as its hint
- **THEN** authoritative Clap parsing still rejects the duplicate input

#### Scenario: Failure exit
- **WHEN** a document operation fails
- **THEN** core exits according to the CLI mapping for that diagnostic class
- **THEN** the failure payload remains owned by diagnostics and output/protocol contracts
