**Target delta.** It bounds Markdown returned-content and visible-selection estimates; `../../design.md` owns the unresolved machine, estimator, and implementation gate.

## MODIFIED Requirements

### Requirement: Markdown pagination and cost use selected output text
Markdown outline, read, and find MUST apply the active pagination budget to selected output text and MUST report cost through shared protocol-compatible cost measurements. Markdown ordinary and nested read MUST report a returned-content estimate only for content returned on the current page and MUST NOT tokenize an unreturned section remainder. Markdown unstructured full-read outline MUST report a returned-content estimate for its returned content. Markdown structured outline MUST enrich only current-page entries with visible-selection estimates derived from cheap existing source spans/facts or an approved bounded input; it MUST NOT serialize, materialize, or tokenize a complete large section solely for cost. Markdown find MUST NOT read, serialize, or tokenize a referenced section solely to report target token cost. Existing Markdown region, ref, Unicode-character pagination, and continuation semantics remain unchanged.

#### Scenario: Read exceeds budget
- **WHEN** a Markdown section exceeds the active limit
- **THEN** read returns bounded content
- **THEN** it exposes the next page value
- **THEN** its returned-content estimate describes only the returned page
- **THEN** Markdown does not tokenize the unreturned section remainder before returning that page

#### Scenario: Nested read uses ordinary read cost scope
- **WHEN** Markdown read is composed as successful unique-ref auto-read
- **THEN** the nested result uses the same returned-page estimate as ordinary read
- **THEN** composition does not trigger complete-section tokenization

#### Scenario: Structured outline includes one large section
- **WHEN** a current-page Markdown outline entry refers to a large section
- **THEN** its visible-selection estimate uses cheap source-span facts or an approved bounded input
- **THEN** cost enrichment does not serialize or tokenize the complete section
- **THEN** entries outside the current outline page are not enriched

#### Scenario: Unstructured full-read returns Markdown content
- **WHEN** navigation selects Markdown unstructured full-read and content is returned
- **THEN** its returned-content estimate describes that returned content
- **THEN** Markdown does not change the full-read content or character-budget contract to satisfy token estimation

#### Scenario: Find returns a Markdown section ref
- **WHEN** Markdown find returns a ref without a composed read
- **THEN** it does not read, serialize, or tokenize the referenced section solely to calculate target token cost
- **THEN** an ordinary or nested read reports its own returned-content estimate if performed
