# Design

本 design 以 Current `OutputSession` shared capability 为执行公约数，分别定义 Entry sequence、text prefix 和 nested auto-read 的 producer-time 组合，并把它们映射到同一个 public output-limit contract。

## Context

### Authority and implementation state

- Stable owner 与 Current code 仍定义 protocol `0.1` pagination：document operation request/closed adapter input 携带 positive integer `page` 和 numeric `limit`，result 携带 page/selection cost，built-in adapters 解释并执行分页。
- [用带单位的输出上限替代分页](../../docs/decisions/product-direction/replace-pagination-with-unit-output-limits.md)是 active、unaligned 的 public future direction；本 Change 获准实施该方向，但该决策本身不是 Current binary evidence。
- [用 Gate、计量策略与 Collector 组合增量输出](../../docs/decisions/product-direction/compose-incremental-output-through-gates-policies-and-collectors.md)是 active、unaligned 的 execution direction。归档的 [introduce-budgeted-output-window](../archive/introduce-budgeted-output-window/design.md) 已让其中的 shared capability 成为 Current：`docnav-protocol` 提供 `CostUnit`，`docnav-text-cost` 提供 requested-unit `TextMeter`，`docnav-output-session` 提供 Gate、InputCost/Projection、Collector、push outcome 和 finish report。
- Current architecture 明确标注：shared Session 尚未接入 `AdapterDocument`、真实 Markdown/JSON producer、CLI 或 protocol wire；归档 Change 也明确要求本 Change 在实施前删除 typed-result 后 traversal/prefix-cropping 模型。
- [保留当前 reference tokenizer](../../docs/decisions/product-direction/retain-current-reference-tokenizer-until-qualified-replacement.md)是 active、aligned 基线。本 Change 使用唯一 `tiktoken-rs / o200k_base / ordinary-text` calculator；backend replacement 不再是依赖或任务。
- `.change-plan.json` 只记录本 Change lifecycle。Proposal、design 和 tasks 是 change-local Target；stable owners 只在实现和验证证据成立后同步为 Current。

### Current and target call chains

Current：

```text
resolved page + numeric limit
  -> AdapterDocument operation
  -> adapter-owned paging/truncation
  -> complete typed OperationResult with page/cost
  -> navigation validation/composition
  -> ProtocolResponse 0.1
  -> protocol-json or readable-view
```

Target：

```text
resolved OutputConstraint
  -> adapter-owned operation producer
  -> OutputSession(Gate + InputCost/Projection + Collector)
  -> typed operation result + OutputReport
  -> navigation validation + optional nested phase + invocation report
  -> ProtocolResponse 0.2 with common output facts
  -> protocol-json or readable-view
```

Producer-time incremental construction is internal. Adapter errors, measurement errors, Collector finish errors and response validation errors return before a complete response is committed; renderer and stdout never consume partial Session state.

### Canonical terms

- `CostUnit`：closed enum `lines | bytes | tokens` shared by public limit, calculators and reports.
- `Limit`：one `CostUnit` plus one positive integer value.
- `OutputConstraint`：normalized internal union `Limited(Limit) | Unbounded`.
- `input atom`：one producer-owned value `I` that Limited Gate either accepts completely or rejects completely.
- `measurement projection`：operation-owned mapping from one input atom to the text measured by the selected unit.
- `OutputReport`：Session-internal Gate/complete facts for one producer phase.
- `invocation output facts`：navigation-owned aggregation of the base phase and any successful nested phase, mapped once to the public success sidecar.

### Scenario obligations

| Success path | Input atom and measurement | Completion obligation |
| --- | --- | --- |
| Structured outline / find | One complete public `Entry` is atomic. InputCost measures that Entry's canonical compact protocol JSON object; array delimiters and root envelope are excluded. | Source exhausted means every selected Entry was admitted. A rejected Entry or unrequested tail makes the base phase incomplete. |
| Read / unstructured outline | A text-prefix producer uses the selected content string and requested unit to form one proven-fitting UTF-8 prefix; measurement covers content only. | Full selected text yields complete; a shorter or empty prefix yields incomplete while preserving fixed result identity/facts. |
| Nested auto-read | Navigation first finishes the base result, derives eligibility from the admitted current result, then runs the existing read behavior with the remaining constraint. | Public complete is false when admitted base content is incomplete or an eligible successful nested result is omitted/truncated by budget. |
| Unbounded content operation | The same producer and Collector run with `UnboundedGate`; no InputCost is constructed. | Producer must exhaust. A successful unbounded invocation maps to `complete:true`. |
| Info / failure | No content Session and no ordinary output sidecar. | Existing success/failure contract remains authoritative. |

## Goals / Non-Goals

### Goals

- Replace the complete public pagination surface with one closed, unit-bearing output constraint and an explicit unbounded branch.
- Integrate real Markdown/JSON operation producers with the Current shared Session without introducing a global reflected field traversal layer.
- Give Entry sequence, text prefix and nested auto-read explicit, independently testable input/measurement/Collector policies while sharing Gate/report semantics.
- Keep protocol-json and readable-view downstream of one complete, validated response and one invocation-level output fact owner.
- Perform protocol/config/CLI/adapter/schema/docs/test/release migration as one incompatible `0.2` cutover.

### Non-Goals

- Replace or dynamically select tokenizer backends.
- Migrate fast-read threshold probing unless implementation uncovers a compile-time API conflict; that active Draft remains a separate consumer.
- Stream protocol or readable output, expose a public Session/cursor, or preserve continuation state across invocations.
- Make limit equal final serialized bytes, renderer lines, billing tokens, process memory or host resource limits.
- Guarantee maximum possible budget fill, introduce an emergency product ceiling, or fail only because a positive limit is small.
- Budget `info`, failure envelopes, fixed protocol/root metadata, readable framing or invocation log content.

## Decisions

### D1. Public input uses one closed constraint union

The three input surfaces map to the same internal state:

```text
machine:
  limit: { unit, value }
  XOR ignore_limit: true

CLI:
  --limit <unit>:<positive-integer>
  XOR --ignore-limit

config defaults.output_limit:
  { mode: "limited", unit, value }
  OR { mode: "unbounded" }
```

Omitting a caller value materializes the core-authored `tokens:6000` preset. Unknown unit, zero/negative/overflow value, extra union fields, both branches, null, or an operation-inapplicable output constraint fails during source resolution or protocol semantic validation before adapter dispatch.

`docnav-protocol` owns the shared Rust `Limit` and `OutputConstraint` identities used by protocol construction, navigation and adapter contracts. Source-specific CLI/config/machine representations map into them but do not become additional domain types; only the request/output wire views are serialized.

`OutputConstraint` is execution control, not adapter selection data. It does not change pathname routing, adapter identity, ref grammar, path-rule selection, fast-read strategy or auto-read mode.

### D2. Protocol `0.2` is an incompatible atomic cutover

Request, result, adapter closed input, config, help and stable documentation remove page, next-page and continuation. Runtime accepts only the `0.2` request shape for document protocol execution; it does not route `0.1`, translate numeric-only limit, alias old config paths, or retain a paging adapter behind the new surface.

Legacy `--page`, `--pagination`, numeric-only `--limit`, `defaults.pagination.*` and `0.1` pagination request fields fail with migration guidance toward a unit limit or explicit unbounded request. Guidance does not constitute compatibility acceptance.

### D3. Content operations hand normalized control to an object-safe producer boundary

`OutlineInput`, `ReadInput` and `FindInput` retain document/selection facts and adapter options but remove page and numeric limit. Navigation passes the already normalized `OutputConstraint` as separate execution control to the matching content operation. `InfoInput` and `info` remain outside this path.

The adapter contract stays object-safe: each concrete content-operation method accepts its prepared semantic input plus the closed constraint and returns its concrete typed result together with an `OutputReport`. The exact Rust wrapper name is implementation-local; its semantic shape is fixed:

```text
(prepared content-operation input, OutputConstraint)
  -> Result<(typed operation result, OutputReport), AdapterError>
```

Navigation authoritatively selects Limited or Unbounded from resolved input. The operation chooses input type, producer, InputCost/Projection and Collector, then instantiates the corresponding shared Session. Adapters do not resolve sources, redefine units, inspect public protocol wrappers or implement a parallel budget state machine.

### D4. Input atoms preserve stable scenario differences

There is one shared Gate/report contract, not one universal business input:

1. Structured outline and find use the shared public `Entry` as the atom. An Entry is accepted unchanged or omitted unchanged; label/summary truncation and “always include one item” soft overflow are removed.
2. Entry InputCost measures the canonical compact JSON encoding of the complete Entry object. Each atom is measured independently and accepted costs are summed; collection brackets/commas, protocol field names outside the Entry, root metadata and presentation framing are excluded. Consequently `lines` counts one compact Entry as one line, while bytes/tokens reflect the complete atomic Entry payload.
3. Read and unstructured outline use a text-prefix input produced from the selected content. `ref`, content type, unstructured reason and other fixed root facts are Collector inputs or constructor facts outside measurement; only returned content text consumes budget.
4. Operation-specific Collectors directly form `StructuredOutlineResult`, `FindResult`, `ReadResult` or `UnstructuredOutlineResult`; the shared Session never fixes `Vec<I>` as the only intermediate representation.

The canonical Entry InputCost belongs to `docnav-adapter-contracts`, where the shared public Entry and standard operation boundary already meet; `docnav-output-session` remains unaware of Entry fields. Canonical Entry encoding is an InputCost policy, not the final response serializer. Serialization/measurement failure uses the Session error channel and prevents a partial result from reaching navigation.

### D5. Text prefix is a shared exactness contract owned by text-cost

`docnav-text-cost` gains a bounded-prefix API over `(CostUnit, &str, limit)` with these invariants:

- returned end offset is a UTF-8 boundary and returned content is an exact prefix of the selected text;
- recounting the returned prefix with the existing calculator for the selected unit equals the returned cost;
- returned cost is no greater than limit;
- complete is true exactly when the full selected text is returned;
- the result is deterministic for the same unit/text/limit;
- empty prefix is legal when no non-empty boundary is admitted, and does not create a special failure.

Bytes and lines may use direct boundary/state logic. Tokens continue to use the unique `o200k_base` ordinary-text calculator; an exact full-count fallback is acceptable before any optimization. The public contract does not promise billing equivalence or maximum theoretical fill, but focused workload evidence must reject a degenerate always-empty implementation and show acceptable resource behavior for representative Markdown/JSON selections, Unicode and adversarial whitespace.

The operation pushes the chosen prefix through a text InputCost so the Session report remains the cost fact owner. Prefix selection and Session measurement must agree; mismatch is an internal error, not a public incomplete success.

### D6. Session reports map once to common public output facts

Affected success adds a closed `output` union at the success-envelope level:

```text
limited:
  mode: "limited"
  limit: { unit, value }
  cost:  { unit, value }
  complete: boolean

unbounded:
  mode: "unbounded"
  complete: true
```

Limited public limit is the original invocation limit, not a per-phase remainder. Public cost uses the same unit and equals the sum of accepted atom costs across the base and successful nested phases. It is not the serialized response size. Unbounded has no selected unit and performs no measurement, so it exposes no cost.

Existing read/unstructured full-selection common `cost` is removed rather than reinterpreted. Entry-local `cost` remains an optional field inside the atomic Entry and therefore participates in that Entry's canonical object cost; it never controls admission independently.

### D7. Navigation composes base and nested phases from one invocation budget

Base content always runs first. After base Session finish and typed-result validation, navigation derives success-only unique-ref eligibility from the admitted current result, preserving the existing current-result rule without scanning omitted producer tail.

- Limited with remaining budget invokes nested read using the same unit and the exact remainder. Its accepted cost is added to base cost; its content is attached only after nested typed-result validation.
- Limited with an eligible nested read and zero remaining budget omits the optional nested payload and marks invocation complete false.
- A budget-truncated nested read may be attached with its valid prefix and makes invocation complete false.
- Base incompleteness always keeps invocation complete false, whether or not admitted entries produce a nested candidate.
- Existing non-budget nested failure remains success-only fallback to the base result. It adds no cost and is not converted into budget incompleteness.
- Unbounded uses the same two-phase composition with no measurement and must exhaust every successful producer phase.

Navigation validates unit equality, checked cost addition, `used <= original limit`, phase completion and final response shape. It owns invocation aggregation but does not remeasure content.

### D8. Fixed metadata is outside the ordinary content budget

Protocol envelope fields, operation discriminator, request-id, request ref echo, content type, unstructured reason, common output sidecar and other required root identity facts remain present even when content is empty. These fields are outside InputCost; therefore a Limited response can serialize to more bytes or lines than its numeric limit.

An accepted Entry is content, not fixed root metadata, so its complete canonical object participates atomically. Readable headers and framing remain presentation-only and are not measured. Invocation logs remove legacy page/pagination facts and may record normalized constraint provenance according to their owner, but do not copy response content or become a second output-cost owner.

### D9. Unbounded is a direct measurement bypass

`ignore-limit` resolves to `OutputConstraint::Unbounded`. The operation constructs `UnboundedGate` without an `InputCost`, consumes the producer to exhaustion and forms the complete typed result through the same Collector family.

Unbounded is not represented by maximum integer, hidden ceiling, default unit, emergency threshold or a Limited report with special values. Allocation, parser, adapter, serialization, renderer and writer failures remain failures; they are not rewritten as incomplete success.

### D10. No public continuation is reconstructed

Incomplete output provides mode, original limit, actual accepted cost and complete=false, but no cursor or resume point. A caller narrows ref/query scope, increases limit, or repeats with unbounded intent. Adapters may keep invocation-private iterators while executing one request, but no iterator position crosses the public boundary or survives the invocation.

### D11. Stable owners become Current only after end-to-end evidence

Implementation first changes shared types, input resolution, adapter execution, built-in producers, navigation composition, protocol/output and verification artifacts under this change-local Target. After focused tests, real CLI/package behavior, schema/examples and full workspace verification agree, stable owners are rewritten from Current pagination to Current OutputSession-backed limits in the same release change.

Decision records are inputs, not progress trackers. This Change may report whether the two active future directions have become fully aligned, but changing decision lifecycle/alignment requires separate explicit authorization under `decision-records`.

## Risks / Trade-offs

- The `0.2` hard cutover breaks old protocol, CLI and config callers in one release. It removes runtime duality but requires schema, help, examples, fixtures and package artifacts to move atomically.
- Removing continuation means an incomplete caller may repeat parsing/search work. The simpler public model trades resumability for one explicit output-control concept.
- Changing `AdapterDocument` content-operation signatures affects shared contracts, two built-in adapters, registry fakes and navigation tests. Object-safe concrete operation returns avoid a generic trait-object framework but still require broad coordinated edits.
- Canonical compact Entry JSON gives an exact, adapter-independent atomic projection and makes `lines` behave as entry count; it intentionally does not equal either complete protocol JSON size or readable rendering size.
- Exact token prefix on the retained backend may perform full-count fallback work. Correctness and deterministic output take precedence; profiling may optimize the algorithm later without changing prefix/report invariants.
- Producer-time stop bounds result construction and avoids producer tail, but does not guarantee the parser, document model or selection setup is lazy. Further producer optimization requires measured evidence and remains outside this contract.
- Nested auto-read uses admitted current-result uniqueness, matching existing result-local semantics. An incomplete base can therefore still expose a nested read for its admitted unique ref, while public complete remains false.
- Fixed metadata outside budget allows total serialized output to exceed the numeric limit. This is necessary to retain a valid success contract and is explicitly not a transport-size guarantee.

## Open Questions

无。Prefix algorithm、concrete Rust wrapper names and adapter iterator organization may vary only within the invariants above; they do not reopen product shape, compatibility, ownership or acceptance semantics.

## Implementation Observations

This section records evidence observed while rebasing the implementation-stage Change. It does not redefine Current stable owners.

### 实施启动交接

当前 Change 已停在第一处代码改动前的交接边界：Readiness 审计全部闭合，Implementation 与 Verification 尚未开始。规划、架构选择、owner handoff 和测试起点已经具备；这里没有待补的产品决策，也不需要再执行一次 Change 生命周期转换。

下一步直接从 `tasks.md` 的 `1.1` bounded-prefix contract 开始改代码，并按 `1.1` → shared protocol/input → adapter producers → navigation/output → end-to-end evidence 的依赖顺序推进。开始修改测试前，仍须按项目规则重新运行完整测试实体与 Case 映射起点检查；当前记录的 577 个 entities / 161 个 Cases 只是本次准备审计的形成时基线。

在 `2.1`–`2.8` 全部通过前，不同步 stable owner；在没有 `decision-records` 明确授权时，不改变 decision lifecycle/alignment。当前准备动作没有修改产品代码、稳定规范、schema、示例、测试、release artifact 或长期决策。

### Rebaseline evidence

- Current source contains `CostUnit`, requested-unit `TextMeter`, `OutputSession`, Limited/Unbounded Gate, operation-neutral InputCost/Projection, Collectors and `OutputReport`. Focused `docnav-protocol`、`docnav-text-cost` and `docnav-output-session` tests pass: 47 tests, 0 failures.
- The archived capability Change is 25/25 complete and explicitly proves shared composition only. Current CodeGraph still finds `PaginationConfig`, `effective_limit`, `PageableEntry`, adapter paging functions and protocol result page fields; it finds no `OutputConstraint`.
- Current bounded `TextMeter` measures complete projected input and can prove threshold exceed; it does not return a UTF-8 prefix end. Bounded prefix is therefore this Change's first new shared implementation task, not an unfinished adjacent Change.
- `bun run test-evidence -- check --root .` passes at this rebaseline: 577 current entities (437 Cargo, 111 Bun, 29 smoke) are mapped by 161 Cases across 12 topics.
- `bun run smoke:docnav` passes 68 development CLI commands. `bun run smoke:docnav-package` still fails at existing `CORE-CONFIG-PATH-002` because the canonical package emits a protocol envelope where that removed config-editor diagnostic expects readable JSON without `protocol_version`; final package evidence must close or correctly migrate this case.

### Test evidence disposition

| Evidence group | Planned treatment |
| --- | --- |
| `WB-PROTOCOL-COST-UNIT-001`, `WB-OUTPUT-SESSION-001` | Keep as Current shared capability evidence; do not broaden them to claim real document integration. |
| `WB-TEXT-COST-001` | Extend with bounded-prefix UTF-8/recount/determinism and representative-resource evidence under the existing text-cost owner. |
| `WB-CORE-ARGS-001`, `WB-CORE-PARAMETER-CATALOG-001`, `WB-NAV-INPUT-RESOLUTION-001`, `BB-CORE-CONFIG-001`, `BB-CORE-TOOLS-001` | Rewrite around the closed constraint union, source precedence, built-in preset, legacy rejection and inspect/help facts. |
| `WB-CONTRACTS-REF-CONFORMANCE-001`, `WB-JSON-READ-001`, `WB-JSON-FIND-005`, `WB-NAV-AUTO-READ-001` | Preserve their independent ref/selection/composition purposes and replace paging assertions with producer-time Session/report observations where that owner can observe them. |
| `WB-JSON-PAGING-002`, `WB-MD-PAGE-001`, `WB-MD-PAGE-002` | Delete when their paging production entities are removed; history does not create replacement obligations. |
| `WB-PROTO-BASIC-001`, `WB-PROTO-DECODE-001`, `WB-PROTO-SCHEMA-001`, `WB-READABLE-VIEW-001`, `WB-OUTPUT-READABLE-MAPPING-001` | Rewrite for `0.2` input/output unions, common facts, raw/readable parity and scope exclusions. |
| New `WB-OUTPUT-LIMIT-INTEGRATION-001` | Add only after direct entities exist; prove real Markdown/JSON Entry/text producers, Limited stop, Unbounded bypass, report mapping and no partial response. |

Before modifying any listed test or Case, rerun the project-required full starting check, query the current Case, and apply the owner/observable-result rules from testing and Case-maintenance docs. Planned IDs in this Change do not create empty current Cases.
