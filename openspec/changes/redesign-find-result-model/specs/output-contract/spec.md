**This provisional delta requires readable find output to project the selected raw model without selecting that model or changing Current presentation before approval.**

## MODIFIED Requirements

### Requirement: Selected renderer owns presentation text

The selected renderer MUST return one complete UTF-8 `String` or `RenderFailure` before the first stdout write. Output orchestration MUST write a successful string exactly as returned without adding framing、separators or a trailing newline. The built-in renderer MUST preserve the repository-owned `readable-view` text contract; a custom renderer owns its own presentation contract. For find, a renderer MUST derive item/group display, evidence, multiplicity, completeness, and continuation presentation only from the finalized protocol facts; it MUST NOT group, deduplicate, parse refs, select representative occurrences, or invent completeness.

#### Scenario: Built-in renderer applies readable-view framing

- **WHEN** the built-in renderer emits a configured content block
- **THEN** its header、block reference and delimiters follow the readable-view contract

#### Scenario: Custom renderer controls its text

- **WHEN** linked code supplies a custom renderer and rendering succeeds
- **THEN** stdout equals the returned UTF-8 string

#### Scenario: Find display preserves identity and evidence boundaries

- **WHEN** the finalized find protocol result contains opaque ref identity and occurrence, representative, or grouped evidence
- **THEN** readable output follows the approved presentation or omission rule for `ref`, `label`, `kind`, `location`, `summary`, `excerpt`, `rank`, `cost`, and `metadata`
- **AND** it does not treat any display-oriented field or ref spelling as an undisclosed alternate identity
- **AND** protocol-json retains the unchanged raw facts

#### Scenario: Renderer cannot complete partial groups

- **WHEN** the finalized find protocol facts mark evidence, count, or a group as partial
- **THEN** readable output preserves that status
- **AND** it does not scan source, fetch another page, or present the fact as complete

#### Scenario: Unapproved model keeps Current projection

- **WHEN** the find model and field roles have not been explicitly approved and finalized
- **THEN** the built-in renderer retains the Current occurrence-oriented find projection
- **AND** no output-only grouping or distinct-ref/node projection is introduced

### Requirement: readable-view maps successful auto-read deterministically

The built-in renderer MUST preserve the documented base outline fields and the finalized base find logical-result fields. When `auto_read` is present, it MUST add a readable `auto_read` object and use `/auto_read/read/content` as the nested content block pointer. The renderer MUST present the auto-read selected by navigation and MUST NOT recalculate current-page or query-global uniqueness.

#### Scenario: successful auto-read uses a nested block
- **WHEN** the response contains `auto_read`
- **THEN** the readable header maps reason, nested read ref, content type, cost summary and page from the protocol result
- **AND** replaces nested content with a block reference at `/auto_read/read/content`
- **AND** emits exactly one length-delimited block with that pointer and the original content bytes

#### Scenario: absent auto-read preserves the base projection
- **WHEN** the response contains no `auto_read`
- **THEN** the readable header uses the documented base outline or finalized find projection
- **AND** no auto-read header field or content block is emitted

#### Scenario: unstructured outline keeps its base content block
- **WHEN** unstructured outline returns its existing base response
- **THEN** its content remains at `/content`
- **AND** no auto-read header field or block is emitted

#### Scenario: Output does not reinterpret auto-read scope
- **WHEN** navigation returns a find response with or without `auto_read`
- **THEN** the built-in renderer projects that immutable response
- **AND** it does not inspect hidden occurrences, later pages, group completeness, or exact-ref multiplicity to change the selection
