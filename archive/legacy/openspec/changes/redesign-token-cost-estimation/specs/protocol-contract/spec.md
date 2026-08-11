**Target delta.** It owns machine-visible approximation and scope requirements; `../../design.md` owns the unresolved encoding, migration, estimator, and implementation gate.

## MODIFIED Requirements

### Requirement: Protocol facts are structured before display
Protocol result fields MUST expose machine-readable facts instead of relying on display strings for semantics. Readable output MUST derive any display text from those facts or from adapter-owned presentation hooks. A public token-cost fact MUST be machine-identifiable as approximate and MUST identify whether it is a returned-content estimate or a visible-selection estimate. Ordinary `ReadResult`, `AutoReadResult.read`, and unstructured `OutlineResult` MUST expose a returned-content estimate for the content actually present in that result. Every structured-outline entry on the current returned page MUST expose a visible-selection estimate derived from cheap existing facts or an approved bounded input; producing that fact MUST NOT require complete target serialization or tokenization, and entries outside that page MUST NOT be measured. A find item MUST NOT claim token cost for referenced target content unless an ordinary or nested read actually returns that content. These token facts MUST NOT change the existing character-pagination or continuation contract.

#### Scenario: Cost facts
- **WHEN** an operation reports cost
- **THEN** protocol output exposes structured cost measurements
- **THEN** readable output may render a compact cost summary from those measurements

#### Scenario: Navigation item facts
- **WHEN** outline, find, or info returns items
- **THEN** protocol output includes stable item facts owned by the operation
- **THEN** display text remains an output-layer convenience

#### Scenario: Bounded read returns one page
- **WHEN** ordinary read returns content and has an unreturned selection remainder
- **THEN** its returned-content estimate describes only the returned `content`
- **THEN** it does not claim or require measurement of the remainder

#### Scenario: Nested read returns one page
- **WHEN** unique-ref auto-read includes `AutoReadResult.read`
- **THEN** the nested `ReadResult` carries the same returned-content estimate meaning as ordinary read
- **THEN** the base outline or find result does not reinterpret that value as complete-selection cost

#### Scenario: Unstructured outline returns full-read content
- **WHEN** outline returns the unstructured content branch
- **THEN** the result exposes a returned-content estimate for its `content`
- **THEN** the fact remains separate from the policy that selected unstructured full-read

#### Scenario: Structured outline exposes a large visible selection
- **WHEN** a current-page structured-outline entry represents a large readable selection
- **THEN** the entry exposes a visible-selection estimate derived from cheap existing facts or an approved bounded input
- **THEN** producing the fact does not require complete selection serialization or tokenization
- **THEN** entries outside the current returned page receive no token-estimation work

#### Scenario: Find returns a ref without content
- **WHEN** find returns an item carrying a readable ref without a successful nested read
- **THEN** protocol facts do not claim an estimate for the referenced target content
- **THEN** a later read reports its own returned-content estimate

#### Scenario: Pagination remains character-based
- **WHEN** a read or structured list result is truncated by its active character budget
- **THEN** existing page and continuation facts determine how the caller continues
- **THEN** approximate-token facts do not select or change the page boundary
