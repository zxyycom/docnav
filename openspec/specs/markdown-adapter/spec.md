# markdown-adapter Specification

## Purpose
Define Markdown adapter behavior: Markdown format probing, parser boundaries, outline/read/find/info semantics, Markdown ref grammar, display facts, pagination and cost behavior, native options, and declared unstructured full-read support. Core CLI, navigation, protocol, output, diagnostics, and shared ref opacity are consumed capabilities, not Markdown-owned rules.

## Requirements
### Requirement: Markdown adapter provides v0 document operations
The Markdown adapter MUST implement outline, read, find, and info for Markdown documents through the linked adapter contract.

#### Scenario: Supported Markdown document
- **WHEN** the selected adapter is Markdown and the document is supported
- **THEN** outline, read, find, and info are available through the standard document operation flow

### Requirement: Probe recognizes only Markdown format support
Markdown probe behavior MUST identify Markdown support and report unsupported input without claiming non-Markdown format ownership.

#### Scenario: Markdown file
- **WHEN** probe receives a Markdown document path
- **THEN** it reports supported Markdown facts

#### Scenario: Non-Markdown file
- **WHEN** probe receives a document that is not recognized as Markdown
- **THEN** it reports unsupported without parsing it as Markdown

### Requirement: Markdown outline returns flat bounded entries
Markdown outline MUST return document-order flat entries with adapter-generated refs and compact display. Code-fence pseudo headings MUST NOT become entries. When filtering leaves no visible heading entry, outline MUST return the whole-document ref entry.

#### Scenario: Nested headings
- **WHEN** a Markdown document contains H1, H2, and H3 headings
- **THEN** outline returns flat entries in document order
- **THEN** each entry contains a unique Markdown ref

#### Scenario: Code fence pseudo heading
- **WHEN** a fenced code block contains text that looks like a heading
- **THEN** outline does not emit an entry for that text

#### Scenario: No visible heading
- **WHEN** current outline parameters leave no heading entry visible
- **THEN** outline returns the whole-document ref entry
- **THEN** read can use that ref to return the whole Markdown document

### Requirement: Markdown read matches canonical refs precisely
Markdown read MUST parse Markdown-owned refs and return the exact referenced region. It MUST distinguish invalid ref grammar, valid-but-unmatched refs, ambiguous refs, and whole-document refs.

#### Scenario: Heading roundtrip
- **WHEN** a caller passes an outline heading ref to read
- **THEN** read returns the corresponding Markdown section
- **THEN** `content_type` is `text/markdown`

#### Scenario: Duplicate heading path
- **WHEN** a document contains duplicate complete heading paths
- **THEN** outline emits distinct refs
- **THEN** read can locate each region separately

#### Scenario: Invalid grammar
- **WHEN** a ref does not match Markdown ref grammar
- **THEN** Markdown reports an invalid-ref diagnostic

#### Scenario: Valid grammar with no match
- **WHEN** a ref matches Markdown grammar but no current region matches it
- **THEN** Markdown reports a ref-not-found diagnostic

### Requirement: Markdown heading refs use canonical snapshot grammar
Markdown heading refs MUST use a canonical, field-tagged grammar that captures the structural snapshot needed for precise matching without requiring shared layers to parse it.

#### Scenario: Heading ref is emitted
- **WHEN** outline emits a heading ref
- **THEN** the ref includes Markdown-owned structural fields
- **THEN** shared layers still treat the ref as opaque

### Requirement: Markdown find returns bounded readable matches
Markdown find MUST return bounded matches with refs that can be read. Match display MUST preserve readable match context without becoming the machine owner for the match facts.

#### Scenario: Match in section
- **WHEN** find matches text inside a Markdown section
- **THEN** the match includes a Markdown ref
- **THEN** read with that ref returns content corresponding to the match region

### Requirement: Markdown info returns compact format facts
Markdown info MUST return a compact summary of Markdown document facts without exposing parser-internal structures as public contract.

#### Scenario: Info request
- **WHEN** info is called for a Markdown document
- **THEN** the result includes stable summary facts useful for navigation
- **THEN** it does not expose private parser state

### Requirement: Markdown pagination and cost use selected output text
Markdown outline, read, and find MUST apply the active pagination budget to selected output text and MUST report cost through shared protocol-compatible cost measurements.

#### Scenario: Read exceeds budget
- **WHEN** a Markdown section exceeds the active limit
- **THEN** read returns bounded content
- **THEN** it exposes the next page value

### Requirement: Markdown native options are declared adapter inputs
Markdown native options MUST be declared through adapter-owned metadata and consumed by navigation input resolution. Markdown option schema and examples are validation material, not independent semantic owners.

#### Scenario: Markdown option is configured
- **WHEN** a project config provides a declared Markdown option
- **THEN** navigation attributes that source
- **THEN** Markdown receives the typed option value

### Requirement: Markdown supports declared unstructured full-read outline
Markdown unstructured full-read outline support MUST be declared through adapter hook metadata before navigation can use it. Normal structured outline behavior MUST remain unchanged when the policy does not apply.

#### Scenario: Policy triggers unstructured full read
- **WHEN** navigation pre-dispatch selects unstructured full-read for a Markdown document
- **THEN** Markdown supplies the full content through the declared hook
- **THEN** the result is not represented as heading entries

#### Scenario: Policy does not trigger
- **WHEN** unstructured full-read policy does not apply
- **THEN** Markdown uses normal structured outline behavior
