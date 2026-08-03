This temporary OpenSpec proposal defines the approved behavior and remaining dependency audit for JSONC support in the built-in JSON adapter; it is not approved for implementation until the blocking audit in `tasks.md` is complete.

## Why

JSON-with-comments is common in editor and workspace configuration, including files that still end in `.json`, but the Current `docnav-json` adapter accepts only a strict UTF-8 JSON value. Current manifest-native routing already selects it through `.json` and `.code-workspace` suffixes plus exact `.prettierrc` and `.watchmanconfig` filenames without reading document content. Those hints are only selection intent: JSONC grammar, source mapping, navigation, normalization, and diagnostics remain JSON-adapter work.

Choosing a JSONC parser in isolation could accidentally widen the grammar to JSON5, weaken single-document or profile boundaries, corrupt strict-profile JSON, or create a public dialect abstraction that later JSON-family changes cannot safely reuse. Parser selection therefore needs a bounded compatibility audit across adjacent JSON families before implementation, while this change's delivered behavior remains JSONC-only.

## What Changes

- Extend `docnav-json` with one JSONC-capable grammar for every selected document, regardless of `.json`/`.jsonc` pathname or explicit adapter selection. Strict JSON remains a valid subset and retains its canonical `json:#` refs, traversal, pagination, cost, and output behavior; a strict parse failure is not retried in a second mode.
- Accept JavaScript-style `//` comments and `/* ... */` block comments wherever JSON whitespace is legal, plus one trailing comma after the last object member or array element. A block comment closes at the first `*/`; `/*` inside it is comment text and does not open a nested comment. Reject unterminated comments, sources that rely on nested-comment structure and leave invalid tokens after the first closer, `#` comments, multiple/missing commas, and all broader JSON5 syntax.
- Before parser selection, use one representative compatibility matrix and corpus to prove that strict JSON profiles remain generic JSON, JSONC configuration variants use the same bounded grammar, JSON5 and multi-document JSON stay rejected, validation/canonicalization profiles are not implicitly claimed, and CBOR/BSON do not force a shared logical model. These cases constrain the implementation but do not add their pathname hints or semantics.
- Keep one normalized format id `json`. The same descriptor declares source content types `application/json` and `application/jsonc`; manifest hints select the same strategy through Current `.json` and `.code-workspace` suffixes, Current exact `.prettierrc` and `.watchmanconfig` filenames, and the new `.jsonc` suffix. Explicit `docnav-json` selection applies the same grammar to any pathname.
- Preserve the existing adapter-owned safety and fidelity boundaries: one optional leading UTF-8 BOM, complete input, unique decoded object member names, maximum depth `127`, strict JSON number grammar/raw-token fidelity, deterministic source order, and bounded source regions.
- Structured `read` always emits deterministic strict JSON as `application/json`. `find` and unstructured full-read use the original BOM-stripped source, including comments and trailing commas; only a match wholly inside one recorded JSONC comment span overrides Current source-region ownership and maps to the deepest enclosing logical container, falling back to the root.
- Source full-read and info use `application/jsonc` only when the parsed source contains JSONC-only syntax; otherwise they use `application/json`. Both report format id `json`.
- Invalid syntax, trailing input, duplicate members, and depth overflow use the Current `DOCUMENT_CONTENT_INVALID` code and stable JSON reasons. Invalid UTF-8 retains `DOCUMENT_ENCODING_UNSUPPORTED`; parser-library errors remain private; no failure re-enters routing.
- Audit implementation strategies and parser dependencies before choosing one. The comparison includes the Current `serde_json`-backed source-aware model, comment/trailing-comma preprocessing, JSONC-capable parsers, and a bounded in-adapter scanner; no candidate, crate, version, or feature set is preselected by this proposal.

## Non-Goals

- This change does not revise the Current automatic pathname routing, registry lookup, pathname-hint ownership, or no-probe contract; it consumes the owner rules established by the archived `replace-probe-traversal-with-inferred-routing` handoff.
- It does not add pathname hints or profile semantics for JSON-LD, GeoJSON, HAR, webmanifest, notebooks, SARIF, `.code-snippets`, or other adjacent JSON formats; that routing expansion is handed off to `expand-json-adapter-pathname-hints` and does not block this change.
- It does not add JSON5, JSON Lines/NDJSON or another multi-root model, profile validation/canonicalization, remote resolution, CBOR/BSON, schema-aware navigation, arithmetic number semantics, a comment-preserving editor/formatter, or new caller-configurable CLI/config/protocol inputs.
- It does not change the shared protocol envelope, generic readable renderer, JSON ref grammar, or create a second adapter/format identity.
- This planning step does not modify owner docs, code, tests, schemas, examples, decision records, dependencies, or any other OpenSpec change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `json-adapter`: add one audited JSONC-capable grammar, its pathname-hint handoff, source/structured-output behavior, safety invariants, diagnostics, and verification requirements while preserving strict-document semantics.

## Impact

- Eventual implementation may affect `crates/adapters/json`, its workspace dependency declaration, the adapter manifest/registry facts consumed by routing, JSON owner docs/specs, semantic Case ledgers, adapter/core/release tests, and contract examples whose observable format or content-type facts change.
- The public operation and ref surfaces remain unchanged for strict JSON documents. JSONC support broadens accepted syntax, including comments in `.json`, adds `.jsonc` and `application/jsonc` to the existing `json` descriptor, and leaves the exact parser/dependency plus measured cost blocked on audit and approval.
- Cross-change ordering is one-way: this change consumes the Current routing contract and adds only `.jsonc`; `expand-json-adapter-pathname-hints` may later consume the generic grammar contract without adding parser semantics here or blocking this implementation.
