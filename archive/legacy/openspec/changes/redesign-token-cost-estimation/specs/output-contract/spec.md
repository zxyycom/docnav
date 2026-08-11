**Target delta.** It governs readable presentation of approximate-token facts; `../../design.md` owns the unresolved machine, estimator, and implementation gate.

## MODIFIED Requirements

### Requirement: Selected renderer owns presentation text
The selected renderer MUST return one complete UTF-8 `String` or `RenderFailure` before the first stdout write. Output orchestration MUST write a successful string exactly as returned without adding framing、separators or a trailing newline. The built-in renderer MUST preserve the repository-owned `readable-view` text contract; a custom renderer owns its own presentation contract. When the immutable protocol response carries an approximate-token fact, the built-in renderer MUST identify it as approximate and MUST preserve whether it is a returned-content estimate or a visible-selection estimate. The built-in renderer MUST NOT calculate a missing token value, issue another document operation, or present a returned-page estimate as complete-selection cost.

#### Scenario: Built-in renderer applies readable-view framing
- **WHEN** the built-in renderer emits a configured content block
- **THEN** its header、block reference and delimiters follow the readable-view contract

#### Scenario: Custom renderer controls its text
- **WHEN** linked code supplies a custom renderer and rendering succeeds
- **THEN** stdout equals the returned UTF-8 string

#### Scenario: Readable read reports returned content estimate
- **WHEN** ordinary or nested read carries its required approximate-token fact
- **THEN** the built-in renderer makes the approximation clear
- **THEN** it identifies the value as a returned-content estimate rather than complete-selection cost

#### Scenario: Readable outline distinguishes cost scopes
- **WHEN** an outline response carries a returned-content estimate for an unstructured result or visible-selection estimates for structured entries
- **THEN** the built-in renderer preserves the protocol-owned scope and approximation
- **THEN** it does not calculate cost while rendering

#### Scenario: Target token cost is absent from a find item
- **WHEN** a find item carries a ref but no target-content token fact
- **THEN** the built-in renderer does not synthesize one from the ref, label, excerpt, byte count, or another operation
