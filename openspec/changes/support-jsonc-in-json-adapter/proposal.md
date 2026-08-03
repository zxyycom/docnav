This temporary OpenSpec proposal defines the approved behavior and remaining dependency audit for JSONC support in the built-in JSON adapter; it is not approved for implementation until the blocking audit in `tasks.md` is complete.

## Why

JSON-with-comments is common in editor and workspace configuration, including files that still end in `.json`, but the Current `docnav-json` adapter only accepts a strict UTF-8 JSON value and only declares `.json`. The approved Target in `replace-probe-traversal-with-inferred-routing` can select the JSON adapter from manifest-native pathname hints, but routing identity is only a hint: JSONC grammar, source mapping, navigation, normalization, and diagnostics remain JSON-adapter work.

## What Changes

- Extend `docnav-json` with one JSONC-capable grammar for every selected document, regardless of `.json`/`.jsonc` pathname or explicit adapter selection. Strict JSON remains a valid subset and retains its canonical `json:#` refs, traversal, pagination, cost, and output behavior; a strict parse failure is not retried in a second mode.
- Accept JavaScript-style `//` and non-nested `/* ... */` comments wherever JSON whitespace is legal, plus one trailing comma after the last object member or array element. Reject `#` comments, nested/unterminated comments, multiple/missing commas, and all broader JSON5 syntax.
- Keep one normalized format id `json`. Manifest hints select the same strategy through `.json`, `.jsonc`, `.code-workspace`, and the exact JSON filenames approved by the routing predecessor; explicit `docnav-json` selection applies the same grammar to any pathname.
- Preserve the existing adapter-owned safety and fidelity boundaries: one optional leading UTF-8 BOM, complete input, unique decoded object member names, maximum depth `127`, strict JSON number grammar/raw-token fidelity, deterministic source order, and bounded source regions.
- Structured `read` always emits deterministic strict JSON as `application/json`. `find` and unstructured full-read use the original BOM-stripped source, including comments and trailing commas; comment matches map to the deepest enclosing logical container, falling back to the root.
- Source full-read and info use `application/jsonc` only when the parsed source contains JSONC-only syntax; otherwise they use `application/json`. Both report format id `json`.
- Invalid syntax, trailing input, duplicate members, and depth overflow use the predecessor's `DOCUMENT_CONTENT_INVALID` code and stable JSON reasons. Invalid UTF-8 retains `DOCUMENT_ENCODING_UNSUPPORTED`; parser-library errors remain private; no failure re-enters routing.
- Audit implementation strategies and parser dependencies before choosing one. The comparison includes the Current `serde_json`-backed source-aware model, comment/trailing-comma preprocessing, JSONC-capable parsers, and a bounded in-adapter scanner; no candidate, crate, version, or feature set is preselected by this proposal.

## Non-Goals

- This change does not implement or revise automatic pathname routing, registry lookup, pathname-hint ownership, or probe removal; those belong to `replace-probe-traversal-with-inferred-routing`.
- It does not add JSON5, schema-aware navigation, arithmetic number semantics, a comment-preserving editor/formatter, or new caller-configurable CLI/config/protocol inputs.
- It does not change the shared protocol envelope, generic readable renderer, JSON ref grammar, or create a second adapter/format identity.
- This planning step does not modify owner docs, code, tests, schemas, examples, decision records, dependencies, or any other OpenSpec change.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `json-adapter`: add one audited JSONC-capable grammar, its pathname-hint handoff, source/structured-output behavior, safety invariants, diagnostics, and verification requirements while preserving strict-document semantics.

## Impact

- Eventual implementation may affect `crates/adapters/json`, its workspace dependency declaration, the adapter manifest/registry facts consumed by routing, JSON owner docs/specs, semantic Case ledgers, adapter/core/release tests, and contract examples whose observable format or content-type facts change.
- The public operation and ref surfaces remain unchanged for strict JSON documents. JSONC support broadens accepted syntax, including comments in `.json`; the exact parser/dependency and its measured cost remain blocked on audit and approval.
- Cross-change ordering is one-way: this change consumes the final routing contract and pathname-hint representation; the routing change must not acquire JSONC parser semantics or wait on this implementation.
