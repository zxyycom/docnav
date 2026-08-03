This temporary delta specifies the selected JSONC behavior and remaining parser-evidence gate for the existing `json-adapter` capability; task 0 must approve one exact implementation before apply.

## ADDED Requirements

### Requirement: JSON adapter 必须以一个受控 grammar 接受 strict JSON 与 JSONC

Every document selected for `docnav-json` MUST use one JSONC-capable grammar, regardless of whether selection came from `.json`, `.jsonc`, `.code-workspace`, an exact JSON filename, or explicit `--adapter docnav-json`. Strict JSON MUST remain a valid subset and MUST preserve its existing logical value, ref, traversal, raw-number, structured-read, pagination, cost, and wrapper behavior. The adapter MUST NOT use a routing-selected mode, parser-success retry, confidence value, or caller option to choose a grammar.

The manifest MUST retain one normalized format id `json`; it MUST add `.jsonc` to the predecessor's approved `.json`, `.code-workspace`, `.prettierrc`, and `.watchmanconfig` hints without adding a second adapter or format identity.

Outside strings, the grammar MUST accept `//` line comments and non-nested `/* ... */` block comments wherever strict JSON permits whitespace. It MUST accept EOF line comments, LF/CRLF/CR line endings, and one trailing comma after the final object member or array element. Comment markers inside strings MUST remain string data. Unterminated/nested block comments, `#` comments, missing or multiple commas, single-quoted strings, unquoted property names, hexadecimal or leading-plus numbers, leading/trailing decimal points, `NaN`, infinity, multiple roots, and every other JSON5/JavaScript extension MUST be rejected.

#### Scenario: JSONC syntax in a `.json` file is accepted

- **WHEN** automatic routing selects `docnav-json` for a `.json` pathname
- **AND** the document uses only strict JSON plus approved comments or trailing commas
- **THEN** the selected JSON strategy parses it with the one JSONC-capable grammar
- **THEN** navigation does not parse content or pass a dialect into the adapter

#### Scenario: Explicit selection uses the same grammar on any pathname

- **WHEN** the caller explicitly selects `docnav-json` for a pathname with any extension
- **AND** the source satisfies the approved JSONC-capable grammar
- **THEN** the adapter parses the document without automatic pathname routing
- **THEN** no second mode or fallback parser is attempted

#### Scenario: Comment markers inside strings remain data

- **WHEN** a JSONC string contains `// literal` or `/* literal */`
- **THEN** those characters remain part of the decoded string
- **THEN** they are not treated as comments

#### Scenario: One trailing comma is accepted

- **WHEN** a JSONC object or array ends its last member or element with a comma
- **THEN** both automatic and explicit selection accept the document
- **THEN** the result does not depend on a parser default

#### Scenario: Broader JSON5 syntax remains unsupported

- **WHEN** a selected document uses an unapproved extension such as a single-quoted string, unquoted member name, missing/multiple comma, hexadecimal number, unary plus, `NaN`, or infinity
- **THEN** `docnav-json` rejects the document

### Requirement: JSONC 必须复用 source-aware JSON 导航语义与安全边界

After removing at most one leading UTF-8 BOM, the selected JSON strategy MUST decode UTF-8 and validate one complete document. Comments and accepted trailing commas MUST be syntax trivia rather than logical nodes. The logical tree MUST preserve object member source order, array index order, node kinds, decoded member names, raw strict-JSON number tokens, root depth `0`, maximum depth `127`, node count, and source regions needed for navigation.

Each object MUST reject duplicate decoded member names exactly as strict JSON does. Number tokens in JSONC MUST use the strict JSON number grammar and MUST retain their original source spelling for structured read. The adapter MUST build one primary logical tree for traversal, ref resolution, structured read, and source occurrence mapping; auxiliary comment/token spans or an offset-preserving parse view MUST remain bounded and MUST NOT become a second full logical tree.

JSONC outline and read MUST reuse the canonical `json:#<fragment>` grammar. Comments and trailing commas MUST NOT create entries, ref tokens, synthetic members, or new node kinds. Object/array traversal, empty-container behavior, root-scalar behavior, pagination, cost, and every outline/find-to-read ref roundtrip MUST match the strict-JSON contract for the same logical value.

#### Scenario: Comments do not change navigation identity

- **WHEN** strict JSON and JSONC documents have the same logical value but the JSONC document adds comments between tokens
- **THEN** outline returns the same ordered labels, kinds, and `json:#` refs for both documents
- **THEN** comments do not create outline entries

#### Scenario: JSONC preserves raw number spelling

- **WHEN** JSONC contains a grammar-valid number token with comments elsewhere in the document
- **THEN** the private model retains the original number token
- **THEN** structured read uses that token under the same normalization rule as strict JSON

#### Scenario: Duplicate decoded members remain invalid

- **WHEN** a JSONC object contains two member spellings that decode to the same member name
- **THEN** `docnav-json` rejects the document as a duplicate-member failure
- **THEN** no canonical ref is produced for either duplicate

#### Scenario: JSONC retains the depth limit

- **WHEN** a JSONC document has maximum logical depth `127`
- **THEN** the adapter may navigate it
- **WHEN** comments or trivia surround a logical node at depth `128`
- **THEN** the adapter rejects the document for maximum-depth overflow
- **THEN** comments do not affect the calculated depth

#### Scenario: BOM handling is not broadened

- **WHEN** JSONC begins with one leading UTF-8 BOM followed by a valid document
- **THEN** the adapter removes that BOM for parsing and source operations while retaining original byte-size facts
- **WHEN** another BOM occurs in the parsed source
- **THEN** it is not silently removed as additional leading trivia

### Requirement: JSONC structured read 与 source read 必须保持分层

Structured `read` of every document accepted by the JSON strategy MUST emit deterministic valid strict JSON with `application/json`: comments and accepted trailing commas MUST be absent, object member source order and raw number tokens MUST be preserved, container layout MUST use two-space indentation, and scalar escaping, spelling, and newline behavior MUST remain adapter-owned rather than inherit an unspecified dependency default. `ReadResult`, cost, page, protocol envelope, and generic readable framing MUST retain their Current shapes.

`find` MUST search the original BOM-stripped source, including comments and accepted trailing commas, by the Current case-sensitive left-to-right non-overlapping literal rule. An occurrence inside a logical token MUST map to the deepest readable logical value that owns the token. An occurrence in comment or other trivia MUST map to the deepest enclosing object or array; trivia outside every child container MUST map to the root. Multiple occurrences mapping to one ref MUST remain separate.

Unstructured full-read MUST return the BOM-stripped source without deleting comments or accepted trailing commas and MUST measure the actual returned text. JSON info and unstructured full-read MUST retain format id `json`. Source content type MUST be `application/jsonc` exactly when the source contains a comment or accepted trailing comma; otherwise it MUST be `application/json`.

#### Scenario: Structured read normalizes JSONC to strict JSON

- **WHEN** a selected JSONC object contains comments and grammar-approved trailing commas
- **AND** read targets that object
- **THEN** normalized content contains the same logical value as valid deterministic strict JSON
- **THEN** comments and trailing commas are absent
- **THEN** the result uses the existing `ReadResult` shape and canonical input ref

#### Scenario: Source full-read retains JSONC spelling

- **WHEN** unstructured full-read is selected for a JSONC document
- **THEN** source content retains comments, whitespace, string escapes, number spelling, and accepted trailing commas after removing at most one leading UTF-8 BOM
- **THEN** cost measurements describe that returned source
- **THEN** the result uses format id `json` and content type `application/jsonc`

#### Scenario: Find can locate a comment occurrence

- **WHEN** a query occurs in a JSONC comment
- **THEN** source search returns a match for that occurrence
- **THEN** the match has a deterministic canonical ref to the deepest enclosing object or array, or to the root when no child container encloses it
- **THEN** read of that ref returns normalized logical content rather than the comment text

#### Scenario: Comment markers in strings stay searchable source

- **WHEN** a query matches `//` or `/*` characters inside a JSONC string token
- **THEN** find treats them as original string source rather than syntax trivia
- **THEN** the occurrence maps to that string value's ref

#### Scenario: Raw and readable wrappers remain independent

- **WHEN** JSONC outline, read, find, info, or full-read succeeds
- **THEN** `protocol-json` serializes only the existing stable result facts
- **THEN** generic `readable-view` derives its existing framing from the same response
- **THEN** neither wrapper parses comments, rewrites refs, or chooses the JSON grammar

### Requirement: JSONC failures 必须保留 adapter-owned diagnostic 边界

Once routing or explicit intent selects `docnav-json`, invalid UTF-8 MUST use the existing `DOCUMENT_ENCODING_UNSUPPORTED` diagnostic. Invalid JSON/JSONC syntax MUST use `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`; trailing non-whitespace input, duplicate decoded members, and maximum-depth overflow MUST use `DOCUMENT_CONTENT_INVALID` with `JSON_TRAILING_INPUT`, `JSON_DUPLICATE_MEMBER`, and `JSON_MAXIMUM_DEPTH_EXCEEDED`, respectively. Canonical content-invalid details MUST contain only the normalized `path` and stable `reason`.

There is no mode or routing-identity conflict because the adapter exposes one grammar and one format identity. Parser crate types, error variants, recovery traces, raw messages, unstable offsets, duplicate member names, and confidence values MUST remain private. The adapter MUST NOT re-enter pathname routing, retry a stricter or looser parser based on parse success, or dispatch another adapter.

#### Scenario: Selected invalid JSONC does not fall back

- **WHEN** routing selects `docnav-json`
- **AND** the actual document has an unterminated block comment or other grammar violation
- **THEN** the operation returns `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`
- **THEN** pathname routing is not invoked again
- **THEN** no other adapter is attempted

#### Scenario: Parser implementation details stay private

- **WHEN** the chosen parser reports an implementation-specific error
- **THEN** the adapter maps it to the approved project diagnostic
- **THEN** the protocol failure does not expose the dependency's type name, raw message, recovery trace, or confidence

#### Scenario: Explicit selection uses the same grammar

- **WHEN** the caller explicitly selects `docnav-json`
- **THEN** pathname routing is skipped
- **THEN** the same JSONC-capable grammar parses the selected source regardless of pathname
- **THEN** a parse failure does not cause a parser retry or adapter fallback

### Requirement: JSONC support 必须有 strict-regression 与 cross-layer 证据

Before JSONC support is Current, JSON adapter owner docs, the main `json-adapter` spec, semantic Case ledger, unit/integration tests, coverage mapping, core CLI smoke, schema/examples affected by observable format/content-type facts, and release package smoke MUST be synchronized. Evidence MUST distinguish unchanged strict-document semantics from newly accepted JSONC syntax and MUST cover automatic and explicit selection under the final routing contract.

The JSONC corpus MUST cover line/block/EOF comments, comment markers in strings, CRLF and Unicode, malformed/unterminated/nested comments, the approved trailing-comma rule for objects and arrays, every rejected loose syntax, optional/multiple BOM behavior, invalid UTF-8, multiple roots/trailing input, decoded duplicates, strict/raw numbers, depths `127` and `128`, empty containers, root scalars, source regions around trivia, comment/source find mapping, outline/find-to-read ref roundtrips, normalized structured read, source full-read, info, pagination, cost, generic readable output, hostile large comments/lines/nesting, and selected failures without fallback.

Strict JSON documents MUST preserve every existing ref, traversal, source-order, number, pagination, cost, info/full-read, diagnostic, and raw/readable assertion not explicitly revised by this delta. `.json` sources containing the accepted comments or trailing commas MUST be accepted through the same grammar; every broader loose or JSON5 syntax listed by this change MUST remain rejected.

#### Scenario: Strict and JSONC matrices run together

- **WHEN** JSON adapter verification runs
- **THEN** strict JSON positive and negative cases prove unchanged strict-document semantics
- **THEN** JSONC cases prove the accepted syntax and identity/output/diagnostic matrix through the same grammar
- **THEN** no parser-library default supplies an untested public behavior

#### Scenario: Release binary proves one strategy

- **WHEN** release package smoke runs from the packaged core `docnav` executable
- **THEN** manifest/registry output exposes one `json` format identity with the approved pathname hints
- **THEN** automatic and explicit operations navigate strict and JSONC source forms through the same linked `docnav-json` strategy
- **THEN** selected JSON failures do not fall back
- **THEN** the public input inventory remains unchanged
