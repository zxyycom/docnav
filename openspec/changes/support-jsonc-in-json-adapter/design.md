This temporary OpenSpec design defines the evidence and decisions needed for JSONC support in `docnav-json`; it deliberately does not select a parser or authorize implementation.

## Decision Status

The observable behavior in Decisions 1–7 is selected: one `docnav-json` grammar accepts strict JSON plus a bounded JSONC extension, one `json` format identity owns all JSON/JSONC pathname hints, and source/normalized output plus diagnostic behavior is fixed. The parser implementation and dependency remain unselected until every candidate is measured against that same contract. The predecessor routing change must also become Current before implementation. No dependency, production, owner-doc, schema/example, or test change may begin before task 0's blocking audit records both gates.

## Context

Current `docnav-json` accepts one UTF-8 JSON value after removing at most one leading UTF-8 BOM. Its `serde_json 1.0.150`-backed loader builds a private ordered tree with source regions, raw number tokens, decoded unique object member names, a root-at-zero maximum depth of `127`, and one shared model for traversal, ref resolution, structured read, and source occurrence mapping. Structured `read` serializes valid pretty JSON; `find` and unstructured full-read consume the BOM-stripped source.

That implementation shape makes JSONC support more than a parser flag. Any candidate must prove that comments and any accepted trailing commas do not corrupt cursor alignment, member/value regions, raw number spelling, duplicate-member rejection, bounded depth, deterministic traversal, or error ownership.

The approved Target in `replace-probe-traversal-with-inferred-routing` replaces registry-order probes with navigation-private manifest pathname lookup and exact registry selection. Its boundary is one-way: routing selects `docnav-json`, but it does not parse JSONC, construct a document model, pass matched format identity into the closed operation input, or choose a parser dialect. This change therefore uses one adapter grammar rather than an unimplementable routing-selected strict/JSONC mode.

The external JSONC specification defines JavaScript-style comments, treats trailing-comma support as optional, recommends `.jsonc`, and describes `application/jsonc`: <https://jsonc.org/>. Docnav adopts comments plus a single trailing comma in objects/arrays because common editor configuration accepts both; every broader extension remains rejected.

## Goals / Non-Goals

**Goals:**

- Extend the existing linked `docnav-json` grammar to accept strict JSON and the approved JSONC syntax regardless of pathname, without a second adapter or routing-selected mode.
- Keep `outline -> ref -> read`, canonical `json:#` refs, source order, pagination, cost, and generic readable projection stable for logically equivalent strict JSON and JSONC documents.
- Fix grammar, identity/hints, normalization, source-read, safety, and diagnostic contracts precisely enough to reject every unapproved parser extension.
- Select an implementation only after source-backed functional, maintenance, security, license, toolchain, target, dependency-weight, binary-size, startup, and parser-fidelity evidence is recorded.
- Keep raw protocol and readable output wrappers unchanged unless the audit finds an unavoidable contract change and the artifacts are revised before approval.

**Non-Goals:**

- JSON5, loose JavaScript object syntax, schema-aware navigation, arithmetic number semantics, or a comment-preserving editor/formatter.
- A second routing detector, adapter callback, confidence score, public parser option, or fallback to another adapter after selection.
- Changing JSON Pointer/ref grammar, core pagination, generic readable rendering, or shared protocol envelopes.
- Reusing routing state or resolving the separate document-state reuse change.

## Decisions

### Decision 1: One JSON strategy and format identity accept strict JSON plus JSONC

Every document selected for `docnav-json` uses the same grammar, whether selection came from `.json`, `.jsonc`, `.code-workspace`, an exact filename, or explicit `--adapter docnav-json`. Strict JSON is a valid subset; parse success is not a retry or mode-selection signal. The manifest keeps one normalized format id `json` and adds `.jsonc` to the predecessor's approved JSON pathname hints.

This deliberately stops promising that comments in `.json` must fail. It preserves strict JSON documents' logical value, refs, ordering, numbers, structured output, pagination, and diagnostics while covering common JSONC-in-`.json` configuration without passing a dialect through the adapter contract.

### Decision 2: The accepted JSONC grammar is closed

Outside strings, `//` line comments and non-nested `/* ... */` block comments are accepted wherever strict JSON permits whitespace. EOF line comments and LF/CRLF/CR line endings are accepted. One trailing comma is accepted after the final object member or array element.

Unterminated/nested block comments, `#` comments, missing or multiple commas, single-quoted strings, unquoted member names, hexadecimal or leading-plus numbers, leading/trailing decimal points, `NaN`, infinity, multiple root values, and every other JSON5/JavaScript extension are rejected. Comment markers inside strings remain string data. Parser defaults cannot widen this list.

### Decision 3: JSONC parsing and source fidelity remain owned by `docnav-json`

Navigation provides only the selected adapter and closed standard operation input. `docnav-json` acquires the bytes, removes at most one leading UTF-8 BOM, validates one complete document under Decision 2 plus the existing number/duplicate/depth rules, and builds the adapter-private model used by all operations.

Routing ASTs, comments, tokens, error strings, or confidence values do not cross into adapter/protocol types. A routing parse tree or routing-selected dialect is rejected because it couples lifecycle and grammar ownership to the wrong layer.

### Decision 4: One primary semantic tree remains the target model

Strict JSON and JSONC feed the existing logical node model so refs, traversal, structured read, info, and source mapping share one semantic tree. Auxiliary token/comment spans or an offset-preserving parse view may be retained when required, but implementation must not create a second full logical tree merely to recover source order.

The dependency audit must prove that the chosen approach preserves original source offsets, raw strict-JSON number tokens, decoded duplicate rejection, root-at-zero depth `127`, and bounded auxiliary state.

### Decision 5: Structured output is JSON; source output preserves JSONC

Structured `read` always serializes deterministic valid strict JSON as `application/json`; comments and trailing commas never enter that payload. Original member order, raw number spelling, pinned string/scalar serialization, two-space container layout, and current newline behavior remain unchanged.

`find` searches the original BOM-stripped source, including comments and trailing commas. Comment/trivia matches map to the deepest enclosing logical object/array; source trivia outside every child region maps to the root. Unstructured full-read preserves that same BOM-stripped source. Info/full-read report format id `json` and use `application/jsonc` only when JSONC-only syntax occurs in source, otherwise `application/json`.

### Decision 6: Public inputs, refs, pagination, wrappers, and failures stay bounded

JSONC support adds no CLI flag, config key, environment variable, protocol request field, adapter option, ref prefix, or second format id. Existing `json:#` refs remain adapter-owned, canonical, ASCII-safe, and opaque to core. Pagination and generic readable wrappers retain current shapes and ordering.

Invalid syntax uses `DOCUMENT_CONTENT_INVALID / JSON_SYNTAX_INVALID`; trailing non-whitespace input, duplicate decoded members, and depth overflow use the predecessor's corresponding stable JSON reasons. Invalid UTF-8 remains `DOCUMENT_ENCODING_UNSUPPORTED`. There is no mode/identity-conflict failure, parser error leakage, routing retry, or adapter fallback.

### Decision 7: Select exactly one minimal sufficient parser implementation

The zero-new-dependency baseline and every external candidate are compared against one behavior corpus and weight table. Final implementation selects exactly one parser approach and at most one new direct parser dependency; parallel fallback parsers are not allowed. A larger crate is acceptable only if the exact minimal feature set resolves to a small acceptable graph/binary/runtime cost and preserves the contract better than smaller candidates.

An approver must accept the exact crate/version/features or the custom-scanner maintenance burden after license, security, MSRV/toolchain, targets, transitive graph, build, binary/package, startup, source-fidelity, and rollback evidence is recorded. Passing a functional spike alone is not approval.

### Decision 8: Routing handoff is one-way

This change consumes the final Current manifest-native pathname contract from `replace-probe-traversal-with-inferred-routing`. It does not edit that change, make routing depend on JSONC implementation, pass a dialect through operation input, or assign parser semantics to navigation. Implementation remains blocked until the predecessor is Current and this change's probe-era wording has been rechecked.

## Candidate Architecture to Audit

The intended operation flow is:

```text
closed operation input
  -> acquire bytes
  -> remove at most one approved BOM and decode UTF-8
  -> parse one complete source with the closed JSONC-capable grammar
  -> build one ordered source-aware logical tree
  -> execute outline / ref resolution / read / find / info / full-read
  -> return existing protocol result or approved owner diagnostic
```

The audit must compare at least these candidates; the listed versions are discovery seeds observed during proposal drafting, not approved dependencies and must be revalidated when the audit runs.

| Candidate | Initial fit | Weight/status that must be proven |
| --- | --- | --- |
| Current `serde_json` plus an adapter-private, offset-preserving JSONC scanner/neutralization view | Retains the existing `DeserializeSeed`, raw-number, serializer, and logical-tree path; can be zero new third-party dependencies | Dependency delta is zero, but custom grammar/security/maintenance weight is unmeasured; a naive regex or lossy rewrite is unacceptable |
| `serde_json_lenient` (discovery seed `0.2.4`) | Exposes comment and trailing-comma controls close to the Current deserializer API | Direct/transitive duplication, maintenance, license/security, raw-value behavior, duplicate callbacks, cursor/source alignment, feature parity, target builds, size, and startup are unverified; <https://docs.rs/serde_json_lenient/latest/> |
| `jsonc_parser` (discovery seed `0.33.0`) | Purpose-built parser with configurable extensions and AST/token/comment collection | Default looseness must be disabled; order, duplicate members, raw number tokens, spans, depth control, serde/CST feature weight, hashing/security, target builds, binary size, and startup are unverified; <https://docs.rs/jsonc-parser/latest/jsonc_parser/> |
| `json_strip_comments` plus Current `serde_json` (discovery seed `3.1.1`) | Replaces comments/trailing commas before strict parse and may support an offset-preserving view | Its extra `#` comment grammar, trailing-comma behavior, diagnostics, exact byte/line preservation, malformed-comment handling, target builds, dependency graph, size, and startup are unverified; <https://docs.rs/json-strip-comments/latest/json_strip_comments/> |

JSON5 parsers and generic JavaScript parsers are not baseline candidates because their normal grammar is broader than the requested JSONC boundary. They may appear only as explicitly rejected alternatives backed by evidence, not as a shortcut.

For every viable candidate, the weight record must include:

1. exact crate/version/features and `cargo metadata` direct/transitive dependency delta;
2. license/notice, advisories, unsafe usage, maintenance/release activity, adoption evidence, and rollback cost;
3. compatibility with workspace Rust `1.96.0`, edition 2021, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`;
4. clean build cost and incremental release executable/package-size delta against the same baseline;
5. cold/warm `docnav` startup and representative strict/JSONC operation latency, with host/cache/repeat facts;
6. behavior-corpus results for grammar, source spans, order, raw numbers, duplicate names, depth, BOM, malformed/untrusted input, and normalization.

## Contract Matrix to Verify

The dependency/implementation audit must prove each selected contract row rather than inheriting library defaults:

| Surface | Required decision/evidence |
| --- | --- |
| Grammar | Accept Decision 2 comments and one trailing comma; reject every enumerated broader extension under all parser configurations |
| Single strategy | `.json`, `.jsonc`, `.code-workspace`, exact JSON filenames, unknown extensions under explicit selection, and direct strategy calls use one grammar without retry |
| Identity/hints | One `json` descriptor; predecessor hints plus `.jsonc`; exact filename/extension case rules remain routing-owned; info format id remains `json` |
| Content types | Structured read is `application/json`; source info/full-read is `application/jsonc` only when JSONC-only syntax occurs, otherwise `application/json` |
| Normalized read | Always deterministic strict JSON; preserve member order/raw number and pinned serializer behavior while removing comments/trailing commas |
| Source operations | Search original comments/source spelling; map trivia to the deepest enclosing container or root; preserve BOM-stripped full-read source |
| Safety/fidelity | One optional BOM, UTF-8, complete input, root depth `0`, maximum depth `127`, decoded duplicate rejection, strict number grammar/raw token, bounded regions/state |
| Diagnostics | Exact `DOCUMENT_CONTENT_INVALID` JSON reason or existing encoding/path diagnostic; parser evidence remains private and no routing/fallback occurs |
| Verification | Strict JSON positive and negative baselines, JSONC corpus, operation/ref roundtrips, raw/readable outputs, CLI/static registry, schema/example implications, and release-package behavior |

## Protocol and Process-Boundary Effects

- The request/success/failure envelopes, pagination fields, cost shape, entry fields, and `json:#` ref syntax are intended to remain unchanged.
- Raw structured `read` remains a string inside the existing `ReadResult` and is normalized strict JSON with `application/json`.
- Generic `readable-view` continues to render the same `ProtocolResponse` facts. It does not strip comments, parse refs, infer hierarchy, or choose a dialect.
- Source full-read and `find` are adapter operations over the acquired source; JSONC content type and source-occurrence semantics remain JSON-owned.
- Parser errors must be normalized into project-owned diagnostics before crossing the adapter contract. External error types/messages, byte offsets not owned by the public contract, and implementation-specific recovery traces must not be serialized.

## Risks / Trade-offs

- **[Risk] Lenient parser defaults accept more than JSONC.** → Set every grammar option explicitly and keep a negative corpus for single quotes, unquoted names, missing commas, hex, unary plus, `NaN`/infinity, nested comments, and multiple root values.
- **[Risk] Comment removal shifts offsets or changes line numbers.** → Require byte/line-preserving transformation or parse spans tied to the original source; prove find-to-read mapping around comments, CRLF, Unicode, and trailing commas.
- **[Risk] A second parser changes number spelling, member order, or duplicate handling.** → Compare the private model and normalized read against the strict baseline; reject candidates that cannot surface the necessary events/tokens without a second full tree.
- **[Accepted trade-off] `.json` with approved comments/trailing commas becomes valid.** → This is intentional coverage for common config files; keep strict JSON document outputs stable and retain a negative corpus for every broader extension.
- **[Risk] JSONC source is mislabeled `application/json`.** → Structured output is always strict JSON; source info/full-read switches to `application/jsonc` when JSONC-only syntax is observed.
- **[Risk] Comment-heavy or deeply nested hostile input causes excessive memory/CPU/stack use.** → Retain the depth limit, bound auxiliary token/comment state, include large-comment/long-line/nesting cases, and measure representative large documents.
- **[Risk] Active routing deltas conflict with identity wording.** → Treat routing as predecessor for implementation, record the one-way handoff, and rebase this delta after its final Current contract rather than editing the other change.
- **[Trade-off] A zero-dependency scanner reduces supply-chain weight but creates owned grammar code.** → Compare maintenance and adversarial correctness cost alongside package metrics; “no dependency” is not automatically the minimal safe choice.

## Migration Plan

1. Complete tasks 0.1–0.7 as read-only verification/spike work against Decisions 1–8 and record exact dependency/weight evidence inside this change.
2. Obtain explicit approval for one exact parser implementation and, if used, one crate/version/feature set with its accepted maintenance/security/license/target/weight trade-off.
3. Revise artifacts only if measured evidence disproves a selected behavior; rebase wording on the final Current routing owner contract.
4. Complete the blocking artifact audit. Only then restore the test-evidence tree, add failing semantic cases, and synchronize owner docs/spec/schema/example requirements in the approved order.
5. Implement the smallest approved parser/model change in `docnav-json`, proving strict-document behavior first and then the JSONC vertical slice through outline/ref/read before find/info/full-read.
6. Run targeted adapter/CLI/release checks and `bun run verify:docnav-workspace`; inspect raw and readable output separately.
7. Roll back by removing `.jsonc` routing and the JSONC grammar path while retaining one `json` identity. Downgrade makes previously accepted comments/trailing commas invalid, so release notes must not claim transparent rollback.

## Remaining Gates

No observable product decision remains intentionally open in this draft. Implementation is still blocked by two evidence gates:

1. **Parser/dependency evidence:** Which candidate, exact version/features, and implementation shape passes functional fidelity and the complete weight table? Audit evidence must support and an approver must accept the dependency or custom-code maintenance trade-off; exactly one implementation path may be selected.
2. **Predecessor readiness:** Has `replace-probe-traversal-with-inferred-routing` become Current, including `DOCUMENT_CONTENT_INVALID`, manifest hints, and probe deletion, and has this change been rechecked without introducing reverse ownership? If not, implementation remains blocked.
