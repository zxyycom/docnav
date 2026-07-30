**本 delta 只定义新的 project find protocol branch 如何进入现有两条 output plans，并在 readable-view 中保持 document path、opaque ref、局部 failure 和 continuation 可见；它尚未通过阻断审计。**

## ADDED Requirements

### Requirement: Both output plans preserve project find facts

`ProtocolJson` and `Rendered(RenderStrategy)` MUST consume the same immutable project find `ProtocolResponse`. `ProtocolJson` MUST serialize the project result unchanged. The built-in readable renderer MUST derive its project presentation from that response without issuing adapter operations, reconstructing match semantics or inventing renderer-only failures.

#### Scenario: Protocol-json emits one project response
- **WHEN** navigation returns a validated `ProjectFindResult`
- **AND** the caller selects `protocol-json`
- **THEN** stdout contains that one complete find success envelope
- **AND** no renderer is invoked

#### Scenario: Readable-view uses the same project response
- **WHEN** navigation returns a validated `ProjectFindResult`
- **AND** the caller selects `readable-view`
- **THEN** the built-in renderer receives that exact response
- **AND** does not dispatch any adapter operation

### Requirement: readable-view presents project matches and failures without merging identity

The built-in readable renderer MUST expose project scope, next page, each match's normalized `document.path`, complete opaque ref and finalized single-document presentation facts. It MUST expose each local failure's document path, diagnostic code and message. It MUST NOT concatenate path and ref into a new identity, add `auto_read`, or hide a non-empty protocol failure list.

#### Scenario: Project match keeps path and ref separate
- **WHEN** readable-view renders a project match
- **THEN** its header contains the document path as a separate fact
- **AND** contains the complete opaque ref from the nested single-document unit
- **AND** any display text is derived without replacing those machine facts

#### Scenario: Partial project result remains visible
- **WHEN** the project protocol result contains matches and local failures
- **THEN** readable-view presents both collections
- **AND** each failure includes at least document path, code and message

#### Scenario: Empty continuable page remains actionable
- **WHEN** a project result has no matches or failures and a non-null page
- **THEN** readable-view preserves that next page
- **AND** does not describe the query as complete

#### Scenario: Project output has no auto-read content block
- **WHEN** readable-view renders a project find result
- **THEN** it contains no `auto_read` object
- **AND** it emits no `/auto_read/read/content` block
