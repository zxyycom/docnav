## ADDED Requirements

### Requirement: adapter protocol execution MUST use static registry library handles
Document operation protocol request execution MUST dispatch through the adapter library handle selected from the current core release static registry. It MUST NOT require adapter direct CLI, adapter `invoke`, external executables, command paths, or dynamic artifact records.

#### Scenario: Protocol request executes through adapter library handle
- **WHEN** core constructs a protocol request for `docnav outline <path>`
- **THEN** `docnav-navigation` dispatches the request to the selected static registry adapter library handle
- **THEN** stdout/stderr and exit code remain owned by core output/CLI surfaces

### Requirement: protocol request construction MUST preserve raw input immutability
Parameter source resolution MAY derive operation values from CLI argv, config sources, defaults, and owner-declared native option sources, but MUST NOT mutate raw argv, raw protocol envelope, or caller-owned request `arguments`.

#### Scenario: Config fills derived operation value without mutating raw input
- **WHEN** core constructs an outline request whose CLI argv omits `limit`
- **AND** config supplies `defaults.pagination.limit`
- **THEN** request construction may serialize the derived limit into the new protocol request
- **THEN** the raw argv record still omits `--limit`
