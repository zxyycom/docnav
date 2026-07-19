本 spec delta 定义 default-on unique-ref auto-read 的 public CLI/config surface、config inspection、strict applicability、退出行为和 invocation logging 边界。Selection、protocol result shape 和 readable mapping 分别由对应 capability delta 拥有。

## ADDED Requirements

### Requirement: unique-ref auto-read is enabled by default

Core CLI MUST expose `--auto-read disabled|unique-ref` for `outline` and `find`. Project and user config MUST accept `defaults.auto_read` with the same exact values. The built-in default MUST be `unique-ref`.

#### Scenario: outline and find expose the exact mode
- **WHEN** a caller requests help for `outline` or `find`
- **THEN** help includes `--auto-read <disabled|unique-ref>`
- **AND** help identifies `unique-ref` as the built-in default
- **AND** no other auto-read token is advertised

#### Scenario: omitted mode enables unique-ref orchestration
- **WHEN** a caller omits `--auto-read` for `outline` or `find`
- **THEN** core resolves the mode as `unique-ref`
- **AND** projects it to document orchestration

#### Scenario: disabled mode preserves the base command
- **WHEN** a caller passes `--auto-read disabled`
- **THEN** core executes only the existing base operation
- **AND** the success result contains no `auto_read` field

#### Scenario: explicit unique-ref supports both document output modes
- **WHEN** a caller invokes `outline` or `find` with `--auto-read unique-ref`
- **AND** selects either `readable-view` or `protocol-json`
- **THEN** core accepts the invocation and projects the resolved mode to document orchestration

#### Scenario: config inspect recognizes the auto-read field
- **WHEN** selected project or user config contains `defaults.auto_read`
- **THEN** `docnav config inspect` reports the canonical auto-read field and source candidate through its existing config-source projection
- **AND** inspection does not construct a document operation or trigger auto-read

#### Scenario: unsupported command rejects the mode before dispatch
- **WHEN** a caller passes `--auto-read` to `read`, `info` or a non-document command
- **THEN** core returns the existing strict input diagnostic
- **AND** no adapter operation is dispatched

#### Scenario: invalid mode rejects the invocation before dispatch
- **WHEN** a caller passes an auto-read value other than `disabled` or `unique-ref`
- **THEN** core returns `INVALID_REQUEST`
- **AND** no adapter operation is dispatched

### Requirement: silent auto-read preserves root exit and logging ownership

Core CLI MUST keep the root invocation operation as `outline` or `find`. When base success does not produce a successful composed auto-read result, the root response MUST remain the validated base success and the process exit code MUST remain `0`.

#### Scenario: nested read diagnostic is silent
- **WHEN** the base outline or find succeeds
- **AND** the unique-ref nested read does not return a validated success
- **THEN** the root response is the unchanged base success
- **AND** the result contains no `auto_read`
- **AND** the process exits with `0`

#### Scenario: invocation logging keeps one root operation
- **WHEN** invocation logging is enabled for auto-read outline or find
- **THEN** the main operation event remains owned by the root outline or find invocation
- **AND** core does not emit a second top-level read invocation event

#### Scenario: explicit content capture handles successful auto-read
- **WHEN** unique-ref auto-read successfully adds read content
- **AND** invocation content capture is explicitly enabled
- **THEN** the added read content uses the existing hashed content-capture path and event shape
- **AND** the main JSONL event does not inline the content
