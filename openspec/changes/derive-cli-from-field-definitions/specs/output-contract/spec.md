## ADDED Requirements

### Requirement: Document failure projection requires a validated output context

Document failure projection MUST use `readable-view`、`readable-json` or `protocol-json` only after upstream parsing/resolution has produced a valid document output mode. A successfully extracted and canonically validated explicit output MUST establish that context immediately after structural parsing. When explicit output is absent, config/default output MUST establish context only after normal navigation resolution succeeds. Output orchestration MUST project later document failures through that validated mode without inferring a mode from malformed raw argv or partial resolution state.

Command-shape failure、duplicate output、invalid output or any failure before a valid output mode is available MUST remain outside document-mode projection and use the core-owned PlainText failure channel. In the absence of valid explicit output, routing、adapter-selection and field-resolution failures that precede successful normal navigation resolution therefore use PlainText. PlainText is not an additional document output mode.

#### Scenario: Valid explicit output controls a later failure

- **WHEN** explicit `--output protocol-json` is successfully extracted and canonically validated before a later selected-field or adapter failure
- **THEN** output orchestration emits the protocol failure envelope
- **THEN** readable or PlainText framing is not mixed into protocol stdout

#### Scenario: Structural parse failure has no validated output context

- **WHEN** authoritative command parsing fails before candidate extraction completes
- **THEN** core reports the caller-input failure through PlainText
- **THEN** output orchestration does not infer protocol/readable mode from raw argv

#### Scenario: Invalid output cannot select its renderer

- **WHEN** the explicit output candidate fails decode or canonical validation
- **THEN** the failure uses PlainText
- **THEN** the rejected value is not passed to document failure projection

#### Scenario: Config output is unavailable before normal resolution succeeds

- **WHEN** explicit output is absent、config requests `protocol-json` and routing or selected-field resolution fails before producing a valid resolved output
- **THEN** the failure uses PlainText
- **THEN** output orchestration does not project from partial config resolution

#### Scenario: Resolved config output controls a later adapter failure

- **WHEN** explicit output is absent、normal navigation resolution selects config/default `protocol-json` and a later adapter operation fails
- **THEN** output orchestration emits the protocol failure envelope
