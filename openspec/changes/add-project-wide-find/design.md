**本设计说明省略 path 的 `docnav find` 如何在 current project root 内复用最终单文档 find 与最终 inference routing contract，产生有限、可继续、path + opaque ref 分离的项目结果；所有实现仍受 predecessor acceptance 与 dependency approval gate 阻断。**

## Context

Current `docnav find <path> --query <text>` 在 core 规范化一个 file path 后，由 navigation 选择一个 adapter、构造 closed `FindInput` 并 dispatch 一次 single-document find。Current raw request 要求 `document.path`；Current `FindResult` 的 ref 只有在该文档内才有意义。

Project find 引入四类不能交给 adapter 的事实：

1. invocation scope 是 core 已解析的 project root，而不是某个 synthetic document；
2. 跨文档导航 identity 是 normalized document path 与 adapter-owned opaque ref 的 pair；
3. lazy discovery、跨文档 order、project continuation 与 partial failure 由 navigation 统一编排；
4. public project request/result 必须与现有 single-document request/result 形成 closed、可校验且 backward-compatible 的 variants。

本 change 有两个 blocking predecessors：

- `redesign-find-result-model` 最终拥有 single-document logical unit、order/page 和 auto-read seam；本 change 只包装它最终批准并实现的 unit。
- `replace-probe-traversal-with-inferred-routing` 最终拥有 automatic 的一次 inference → normalized format → exact registry match，以及 explicit adapter exact lookup/manifest format facts；本 change 只为每个 project candidate 调用该 seam。

`audit-runtime-performance-boundaries` 独立拥有 runtime measurement、attribution、baseline/budget/gate 和 owner handoff。本文的固定 quantum 是 correctness/continuation boundary，不是 wall-clock threshold。

## Goals / Non-Goals

**Goals:**

- 保持显式 path 的 single-document CLI、raw request encoding、adapter input/result、ref 和 auto-read 行为。
- 让省略 path 的显式 `find` 只搜索 current resolved project root。
- 在 core/navigation 内完成 project-local ignore-aware discovery、predecessor-owned routing、per-document dispatch、fixed-quantum numeric replay 和 bounded partial failures。
- 在 raw protocol 中增加明确的 single-document/project find request closed union；project variant 携带 resolved project root，不伪装 document。
- 在 protocol/readable 中分开保留 document path、final single-document unit 和 opaque ref。
- 只调用现有 single-document adapter strategy，不增加 project-aware adapter operation。

**Non-Goals:**

- 决定或重写 single-document find result model 或 inference implementation。
- 持久 index、cross-run cache、background service、relevance ranking、fuzzy search、query-language extension 或 `fast-find`。
- wall-clock timeout、size-based silent skip、parallel dispatch 或实时 progress。
- symlink following、project-root escape、user-global ignore state 或新的 Docnav ignore language。
- project-mode auto-read 或 cross-document read composition。
- opaque cursor、result-set id、snapshot persistence 或 request-controlled project work tuning。
- 省略 `find` subcommand 的 natural-language/query routing。

## Decisions

### Decision 1: path presence is the only CLI scope discriminator

**Status: confirmed for this change; implementation remains blocked.**

- `docnav find <path> --query <text>` remains single-document find.
- `docnav find --query <text>` selects project find and uses this invocation's resolved `ProjectContext.project_root`.
- A supplied path must retain existing file validation; an explicit directory is not a project alias.
- Query content, cwd file count, path-looking query tokens and adapter outcomes never choose scope.

Optional path preserves the confirmed command surface without creating a second query/options command. Treating a directory path as project scope would silently change an existing document-path failure and is rejected.

### Decision 2: raw find request is a closed backward-compatible union

**Status: confirmed.**

The public raw `find` request type becomes exactly:

```text
FindRequest =
  SingleDocumentFindRequest {
    protocol_version,
    request_id,
    operation: "find",
    document: { path },
    arguments: FindArguments
  }
| ProjectFindRequest {
    protocol_version,
    request_id,
    operation: "find",
    project: { root },
    arguments: FindArguments
  }
```

`SingleDocumentFindRequest` retains the existing serialized field names, required fields and `FindArguments` encoding byte-for-shape; old valid request fixtures remain valid. `ProjectFindRequest.project.root` is the already-resolved normalized project-root string. It has no `document` member. The two branches are closed and mutually exclusive: `document` and `project` cannot be omitted together or supplied together, and no `scope`/synthetic path heuristic repairs an invalid envelope.

The project envelope ends at core/navigation orchestration. For each selected candidate, navigation constructs the existing closed single-document `FindInput`; project root, project page state and failure accumulator never enter adapter input/options. A heterogeneous closed union is smaller and safer than changing `document.path` to mean both file and directory or inventing a fake file identity.

### Decision 3: core owns scope/root; navigation owns project orchestration; adapter sees one document

**Status: confirmed.**

Core continues to own argv/help, project-root resolution, explicit document normalization, parameter-catalog facts, output plan and process exit mapping. It hands raw project scope plus exact resolved root to navigation without reading project documents.

Navigation owns:

1. project-local lazy traversal and ignore evaluation;
2. deterministic traversal position;
3. per-candidate automatic or explicit routing through the predecessor seam;
4. full-config validation followed by selected-adapter parameter projection;
5. closed raw request projection plus existing single-document `FindInput` construction/dispatch;
6. result wrapping, local/fatal classification, fixed-quantum replay and page projection.

Each adapter call receives one normalized document path, query, adapter-owned `limit`, adapter page and applicable typed options. It never receives project root, candidate list, outer page, traversal position, cross-document accumulator, auto-read mode or output strategy.

### Decision 4: traversal is per-directory sorted deterministic DFS

**Status: confirmed behavior; implementation/dependency remains approval-blocked.**

Traversal:

- recursively considers regular files under project root;
- applies project `.gitignore`, nested `.gitignore` and `.ignore`;
- ignores user-global Git ignore/exclude state;
- always excludes `.docnav`, `.git`, `.hg` and `.svn` control directories;
- includes ordinary hidden entries unless a project-owned rule excludes them;
- uses symlink metadata and skips both file and directory symlinks without following them;
- never silently excludes a regular file by size.

For each visited directory, traversal reads only that directory's immediate entries into a directory-local buffer, derives lossless normalized path segments, sorts those entries by case-sensitive UTF-8 bytes, and visits them depth-first in that order. The DFS stack may retain directory-local sibling state needed to resume ancestors, but navigation MUST NOT build or sort a flat all-project candidate list. A directory listing, owned ignore source or identity normalization that cannot support deterministic replay is fatal rather than silently incomplete.

No walker/ignore crate is selected by this design. Apply must compare at least one mature gitignore-aware candidate, alternatives and a no-new-dependency implementation. The evidence table must cover ecosystem adoption, maintenance/release health, security advisories/unsafe/transitives, license/notice, workspace toolchain and MSRV, Linux/Windows targets, minimal features, package/binary size, cold/warm startup, ignore/symlink/order correctness, alternatives and rollback. A named human owner must approve exact crate/version/features or no-new-dependency. Before that approval, Cargo manifests and lockfile MUST NOT change.

### Decision 5: automatic and explicit project routing consume the predecessor's final seams

**Status: confirmed; blocked on `replace-probe-traversal-with-inferred-routing` owner acceptance and completion.**

Before traversal, navigation validates global registry/catalog invariants. A static registry with duplicate/conflicting normalized format identities is a global fatal failure; it is not document-local evidence and registry order cannot choose a winner.

With no explicit adapter intent, each eligible regular-file candidate performs exactly one predecessor-owned inference invocation:

```text
candidate path
  -> inference once
  -> project-owned normalized format identity
  -> exact registry format match
  -> selected adapter real find parse/execution
```

Within one project request/replay, that selected outcome is invocation-local and reused while the same document advances through adapter pages. A later numeric page request reconstructs the route by invoking inference again during its fresh replay; this is deterministic recomputation, not cross-run selection cache.

- The predecessor's exact `Unknown` inference outcome is ordinary filtering.
- A recognized normalized format with no exact registered adapter is ordinary unsupported filtering.
- Candidate-specific inference document I/O failure is one bounded local document failure.
- `FORMAT_AMBIGUOUS` from multiple inferred identities is one bounded local document failure and advances the document.
- An unclassified `format-routing-failed` outcome is one bounded local document failure and advances the document.
- Only a duplicate/conflicting static registry format-identity invariant is routing-global fatal.
- After exact selection, the adapter performs its real acquisition/parse/find. Parse, semantic, operation or selected-result validation failure is one bounded local document failure; navigation never tries another adapter.

With explicit adapter intent, navigation performs exact adapter-id lookup once before traversal and skips inference. A missing id returns the existing `ADAPTER_UNAVAILABLE` / `ADAPTER_NOT_FOUND` missing-id diagnostic. Duplicate adapter ids are rejected by static registry validation before caller lookup, so caller exact lookup is not attempted. The selected definition's final manifest format descriptors are projected to a deterministic path-eligibility prefilter using the descriptor owner's approved extension comparison semantics. Descriptor-ineligible files are ordinary filtering. Eligible files still run the selected adapter's real parse/find; descriptor metadata is not proof of valid content, and selected failure is local. Project find MUST NOT restore candidate-execution routing or invent a project-only format map.

### Decision 6: project result wraps the predecessor unit without changing it

**Status: confirmed; nested unit remains predecessor-owned.**

Project success uses:

```text
scope: "project"
matches[]:
  document:
    path string, required
  match:
    SingleDocumentFindUnit, required
failures[]:
  document:
    path string, required
  error:
    existing ProtocolError projection, required
page:
  positive integer | null, required
```

`SingleDocumentFindUnit` is the exact finalized and implemented unit from `redesign-find-result-model`; this change does not choose occurrence/node/group/evidence/multiplicity fields. `document.path` uses the existing normalized slash-path contract. The adapter ref remains nested, complete and opaque. Shared layers neither prepend project path nor parse/deduplicate equal refs across documents.

One document with no matches produces no wrapper. A document-scoped failure produces at most one failure wrapper. The `(document.path, match.ref)` pair can be passed to ordinary explicit-path `read`, where only that selected adapter parses ref.

### Decision 7: local failures are bounded success facts; global failures stay fatal

**Status: confirmed.**

Local document failure, after a unique document identity exists:

- candidate metadata/open or inference document I/O failure;
- `FORMAT_AMBIGUOUS` caused by multiple inferred identities;
- unclassified `format-routing-failed`;
- selected adapter acquisition/parse/semantic/find failure;
- selected single-document result validation failure.

Each produces at most one existing diagnostic projection and advances past that document. Unknown inference, unsupported normalized format, explicit-descriptor mismatch and a valid no-match result are normal filtering/outcomes, not failures.

Fatal failure includes invalid raw request/argv/query/config/catalog, unresolved/unreadable project root, root/nested directory or owned ignore-source enumeration failure, unrepresentable/colliding path identity, explicit adapter lookup failure, duplicate/conflicting static registry format-identity invariant, project result validation and output preparation. A fatal outcome uses the existing single top-level failure envelope.

Any validated project result—including mixed, failure-only or empty-continuable—maps to success/exit `0`. Putting local failures only on stderr would remove machine facts; emitting sibling envelopes would break one-invocation/one-response, so both are rejected.

### Decision 8: adapter `limit` remains adapter-owned; project pages use one fixed internal quantum

**Status: confirmed as the project continuation owner.**

The resolved positive `limit` keeps exactly its Current meaning: every single-document adapter dispatch receives it as that adapter's result budget. Project orchestration MUST NOT reinterpret it as a discovery, dispatch or outer-result quota.

Project owner defines one positive, finite, non-configurable work quantum. Its exact value is implementation-private: it is fixed within one build so replay is deterministic, but it is not serialized, exposed as CLI/config/protocol input, encoded in public schema/examples, or promised as a compatibility constant across builds.

Before production pagination is implemented, a change-local validation gate must choose the initial private value from representative empty, filter-heavy, multi-page and local-failure workloads and prove it is greater than zero, pages remain finite, every non-fatal state advances, and stable inputs replay identically within the same build. The evidence records the private implementation choice without promoting its exact value into public contract.

One transition emits at most one complete project unit and must advance at least one component of the replay state:

```text
(document_position, adapter_page, logical_unit_offset)
```

- `document_position` is the next deterministic DFS entry position, including entries that become filtered/skipped; advancing it also resets adapter page to `1` and offset to `0`.
- `adapter_page` is the positive single-document find page currently being replayed.
- `logical_unit_offset` is the next finalized unit within that validated adapter page.

The transition machine is closed:

1. A non-file, ignored, unknown, unsupported or explicit-descriptor-ineligible entry advances `document_position`.
2. A bounded local failure emits one failure wrapper and advances `document_position`.
3. A non-empty validated adapter page with a remaining unit emits exactly that complete wrapped unit and increments `logical_unit_offset`.
4. When all units of a continuable adapter page are consumed, the transition sets `adapter_page` to the exact validated returned page and resets offset to `0`.
5. An empty-but-continuable adapter page performs rule 4 without output; it cannot repeat the same state.
6. A terminal adapter page with no remaining unit advances `document_position`.
7. A fatal failure terminates the invocation and emits no project success.

Invocation-local directory/page buffers may avoid repeated work inside one request, but they are derivable from the triple and are not continuation state. Invalid adapter pagination/result becomes a local failure and advances the document rather than looping.

Each project page executes no more transitions, and emits no more match/failure wrappers combined, than that build's private positive finite quantum. This bound does not cap one directory listing's immediate-entry buffer, inference call or adapter call by time/bytes; those costs remain observable owner facts rather than silent eligibility changes.

### Decision 9: numeric page replay reconstructs state; empty continuation is valid

**Status: confirmed.**

Project request page defaults to positive integer `1`. To answer page `n`, navigation starts from `(0, 1, 0)`, deterministically replays the preceding `n - 1` logical page steps with the same build-private quantum while discarding earlier outputs, then executes one bounded logical step for page `n`.

- If the page quantum ends before terminal state is proven, response page is request page `+ 1`.
- If terminal state is reached, response page is `null`.
- Request beyond terminal returns empty `matches`, empty `failures`, `page: null`.
- Empty filtering runs and empty-but-continuable adapter pages may consume the page quantum, so an empty project result with non-null continuation is valid.
- Repeating a request with stable project root, query, adapter intent, options, adapter `limit`, file tree, file contents and project-local ignore state reproduces the same boundaries.
- Mutation between invocations is evaluated as current state; no snapshot claim is made.

There is no opaque cursor, result-set id, persisted traversal state or cross-run cache. Conservative extra empty terminal page is permitted when terminal state was not proven before the prior quantum ended.

### Decision 10: project scope does not auto-read

**Status: confirmed.**

Single-document auto-read remains owned by the finalized predecessor contract. Project result never contains `auto_read`, never computes query-global/composite uniqueness and never dispatches nested read.

Project selected view does not materialize `defaults.auto_read`. An explicit project `--auto-read` is scope-inapplicable and fails before discovery; valid configured values remain recognized by full config validation but do not alter project orchestration.

### Decision 11: response/output add a closed project branch without merging identity

**Status: confirmed.**

Outer operation remains `find`. Explicit-path requests retain the final single-document success branch. Project requests return required `scope: "project"` `ProjectFindResult`.

`ProtocolJson` serializes that immutable response. Built-in readable rendering shows project scope/page, each independent document path, full opaque ref and finalized nested unit facts, plus each local failure's path/code/message. It does not dispatch adapters, concatenate path/ref, add a display-only identity, hide failures or add auto-read content.

The request union and result variant are additive for project callers, but schema/types are closed. Apply must update exhaustive consumers, request and response schemas, examples and decode/validation tests together. Existing single-document request/result examples must continue to validate unchanged.

### Decision 12: both predecessor handoffs are blocking and remain single-directional

**Status: confirmed.**

Implementation order:

1. `redesign-find-result-model` owner explicitly accepts this wrapper handoff and completes final contract, implementation and validation.
2. `replace-probe-traversal-with-inferred-routing` owner explicitly accepts automatic/exact/manifest handoffs and completes final contract, implementation and validation.
3. This change completes dependency/feasibility audits and receives explicit traversal dependency or no-dependency approval.
4. This change synchronizes owner docs, request/response schema/examples and test evidence before production implementation.
5. Core/navigation/protocol/output implementation and workspace verification follow.

Neither predecessor imports project traversal or result ownership. This change does not edit their artifacts or select their dependencies. `audit-runtime-performance-boundaries` may consume measurements at any stage but is not an implementation prerequisite and cannot silently change the fixed semantics above.

## Risks / Trade-offs

- **Predecessor drift:** either final seam may differ from this draft → blocking owner-acceptance tasks must update this change before implementation; provisional fields are not copied.
- **Directory fanout:** sorted DFS buffers one directory's immediate entries → memory may scale with the largest directory and active DFS sibling stacks, but not total project entries; dependency audit must measure and document this.
- **Later-page replay:** page `n` replays `n` bounded logical steps using the same build-private quantum → cost grows predictably within that build; accepted to avoid cursor/cache ownership and remains attributable by runtime audit.
- **Large/slow single operation:** fixed transitions do not bound a directory read, inference or adapter parse by wall clock → behavior remains complete/observable rather than silently skipping; adapter/runtime owner handles optimization.
- **Project mutation:** changed files/ignore rules/content can move page boundaries → contract promises deterministic replay only for stable state, not snapshots.
- **Partial failures overlooked:** exit `0` may be insufficient signal → both protocol and readable output must expose failures; examples/smoke cover mixed and failure-only pages.
- **Descriptor false positive/negative:** explicit manifest prefilter is metadata, not parse proof → predecessor owner must accept comparison semantics; actual selected parse remains authoritative and visible.
- **New dependency risk:** a walker may increase supply-chain, size, startup and platform surface → no Cargo change before comparative evidence and explicit human approval.
- **Closed-union migration:** exhaustive request/result consumers must add project branches → apply updates schema/types/examples/tests atomically while retaining the old document branch.

## Migration Plan

1. Complete both blocking predecessor acceptance/completion tasks; revise this change if either finalized contract differs.
2. Produce change-local traversal dependency audit and obtain explicit exact dependency/version/features or no-new-dependency approval.
3. Prove per-directory DFS, inference/error taxonomy, fixed replay state machine and closed-union compatibility in the blocking audit.
4. Use representative workloads to validate and select the initial positive finite build-private quantum without encoding its exact value in public artifacts.
5. Update long-term owner docs plus raw request/response schemas, typed protocol models, examples and validation fixtures; prove existing document requests still validate.
6. Implement optional-path CLI/root handoff, lazy discovery, final routing seam, per-document dispatch, result wrapper, fixed-quantum replay and output/process mapping.
7. Restore/update test evidence, run scoped Rust/schema/example/CLI checks and `bun run verify:docnav-workspace`.

Rollback removes the pathless CLI branch, `ProjectFindRequest`, `ProjectFindResult` and project renderer. Existing explicit-path request/result and adapter contracts require no data migration, index cleanup or cache invalidation.

## Open Questions

1. **Which exact traversal implementation is approved?** `dependency-audit.md` must compare maintained walker/ignore candidates and no-new-dependency, then a named human must approve exact crate/version/features or the no-dependency path. Until recorded, Cargo manifests/lockfile and production traversal MUST NOT change.
2. **Have both predecessor owners accepted the exact handoffs and completed validation?** Tasks 1.2 and 1.3 record the accepted final logical-unit and routing/descriptor seams. Any mismatch requires revising this design/specs before implementation; an active or merely proposed predecessor is not acceptance.
3. **Which positive finite initial quantum passes the private validation gate?** Task 1.7 selects it from representative workloads and records same-build replay/progress evidence. Its exact value remains implementation-private and does not enter schema, examples or compatibility promises.
