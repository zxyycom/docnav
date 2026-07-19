本 spec delta 定义 base 或 composed `ProtocolResponse` 的两条 public output projection。Protocol result shape 由 `protocol-contract` 拥有；本 delta 只拥有 plan selection、readable mapping、block framing 和 output failure boundary。

## ADDED Requirements

### Requirement: both output plans consume one navigation response

After navigation selects a validated base or composed `ProtocolResponse`, `ProtocolJson` and the built-in `Rendered` plan MUST consume that same immutable response. Output orchestration MUST NOT issue another read or maintain renderer-only selection/failure facts.

#### Scenario: protocol-json serializes successful auto-read
- **WHEN** navigation returns a composed result with `auto_read`
- **AND** the caller selects `protocol-json`
- **THEN** `ProtocolJson` serializes the complete outer response as the only stdout JSON value
- **AND** includes the protocol-owned `auto_read` object unchanged

#### Scenario: protocol-json preserves the base response otherwise
- **WHEN** navigation returns the base response without `auto_read`
- **THEN** `ProtocolJson` serializes that base response without any sibling auto-read metadata

#### Scenario: readable-view derives from the same facts
- **WHEN** a caller selects `readable-view`
- **THEN** the built-in renderer derives its base and optional auto-read presentation from the same response
- **AND** does not invent selection, skipped or failed facts

### Requirement: readable-view maps successful auto-read deterministically

The built-in renderer MUST preserve the existing base outline/find readable fields. When `auto_read` is present, it MUST add a readable `auto_read` object and use `/auto_read/read/content` as the nested content block pointer.

#### Scenario: successful auto-read uses a nested block
- **WHEN** the response contains `auto_read`
- **THEN** the readable header maps reason, nested read ref, content type, cost summary and page from the protocol result
- **AND** replaces nested content with a block reference at `/auto_read/read/content`
- **AND** emits exactly one length-delimited block with that pointer and the original content bytes

#### Scenario: absent auto-read preserves the base projection
- **WHEN** the response contains no `auto_read`
- **THEN** the readable header uses the existing base outline/find projection
- **AND** no auto-read header field or content block is emitted

#### Scenario: unstructured outline keeps its base content block
- **WHEN** unstructured outline returns its existing base response
- **THEN** its content remains at `/content`
- **AND** no auto-read header field or block is emitted

### Requirement: output failures retain existing ownership

Renderer failure and writer failure MUST use the existing output failure boundaries. Auto-read MUST NOT introduce a second output attempt or fallback renderer.

#### Scenario: nested content framing invariant fails
- **WHEN** the built-in renderer cannot resolve or frame `/auto_read/read/content`
- **THEN** it returns `RenderFailure` before the first stdout write
- **AND** output orchestration preserves the existing empty-stdout and no-fallback behavior
