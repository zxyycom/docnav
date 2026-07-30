This design records the mechanism-neutral Target, confirmed responsibility
boundaries, candidate comparison, and unresolved owner decisions for
`reuse-adapter-document-state`. It is not an approved `Session`, handle,
prepared-state, shared-source, or Rust type design.

## Context

Current navigation first selects a linked adapter by calling `probe(path)`, then constructs one closed operation input and dispatches the selected strategy. A structured outline/find success may trigger navigation-owned unique-ref auto-read, which dispatches the same adapter's ordinary `read` strategy. Outline policy may instead call a cost-measurement hook followed by an unstructured full-read content hook, or may fall back to the structured outline strategy.

Current Markdown and JSON adapters acquire and decode the path separately in probe and in every operation/hook. Each Markdown operation and full-read hook constructs a new `MarkdownDocument`; JSON probe and every JSON operation/hook each construct a new adapter-private `JsonDocument`. Consequently:

| Current path | Selected-adapter preparation work |
| --- | --- |
| direct outline/read/find/info | probe acquisition/decode (and JSON parse), then operation acquisition/decode/parse |
| cost threshold does not select full read | probe, cost hook, then structured outline each prepare independently |
| cost threshold selects full read | probe, cost hook, then content hook each prepare independently |
| eligible unique-ref auto-read | probe, base outline/find, then nested read each prepare independently |
| automatic discovery | each unsupported/invalid candidate prepares as its probe requires; the selected candidate then prepares again for dispatch |

This is not a request to merge operations. Navigation must continue to decide whether and how operations compose, validate the base result, apply Current fallback policy, and own the invocation lifetime. Adapter-private format detection, source decode, parser tree/index, source regions, ref generation/resolution, and result semantics must remain with the adapter.

Stakeholders are the navigation and adapter-contract maintainers, Markdown and JSON owners, new adapter authors (notably the code adapter), interactive composition consumers, local-service maintainers, and the architecture/product owner who must approve source-view semantics and the reuse boundary.

## Planning Interpretation

- **Current** statements describe behavior that must be checked against owner
  docs and implementation evidence. This change is not evidence that Current
  behavior exists.
- **Target** statements define the outcome and compatibility boundaries this
  change intends to establish.
- **Confirmed** decisions constrain every candidate but do not select a
  concrete representation.
- **Open** decisions are outputs of the architecture/product-owner gate in
  tasks 1.1–1.7. They cannot be filled with agent defaults.
- **Approved invocation lifecycle** and **approved document view** mean the
  exact mechanism, stage lifetime, source snapshot, refresh, cleanup, and
  fallback rules that the owner will approve. No such definition exists yet;
  task 1.8 must replace the mechanism-neutral wording with the complete
  approved rules before implementation.

`proposal.md` owns overall change status and scope. `tasks.md` owns the gate and
apply sequence.

## Goals / Non-Goals

**Goals:**

- Prevent composition alone from causing repeated complete acquisition, decode, or parse of the same selected-adapter document view within one navigation invocation.
- Cover direct operations, declared and automatic adapter selection, unstructured full-read selection/content/fallback, and unique-ref nested read rather than optimizing only one happy path.
- Keep navigation in control of composition, failure fallback, and lifetime; keep all format-specific state and algorithms owned by the adapter.
- Make snapshot/TOCTOU, unsupported-candidate cleanup, cancellation/failure cleanup, and future process-boundary consequences explicit before implementation.
- Preserve public protocol, ref, output, pagination, CLI, and auto-read result/fallback behavior.
- Require count-based and mutation-based evidence rather than treating a new trait or object as proof of reuse.

**Non-Goals:**

- Selecting a request-scoped or operation-shaped session before the candidate gate.
- Reusing state across invocations, caching by path, retaining state in local service process caches, or defining invalidation for a long-lived document cache.
- Sharing parser trees, resolved nodes, refs, source-region indexes, or format error types across adapters or with core.
- Adding state identifiers, opaque handles, snapshot metadata, or cleanup status to the public protocol, output, ref, continuation, log, schema, or example surface.
- Changing automatic discovery order, probe support meaning, Current auto-read eligibility, nested-read silent fallback, full-read result shape, or direct operation results.
- Choosing token-cost producers/sinks or any find occurrence/node/grouped model.
- Defining an external adapter runtime, adapter wire session, or local-service document cache.

## Responsibility and Lifetime Map

The word “ownership” has three independent meanings in this change:
responsibility for behavior, storage/allocation of source or parser values, and
control of when a value remains reachable. The gate must decide each axis
explicitly rather than inferring all three from one Rust type.

| Concern | Confirmed owner or decision state | Lifetime / visibility |
| --- | --- | --- |
| Navigation composition | `docnav-navigation` decides adapter selection, direct dispatch, full-read policy/fallback, unique-ref nested read, validation, and which stage runs next. | Navigation bounds the reuse opportunity to one invocation and decides when no later eligible stage exists. |
| Adapter format semantics | The selected adapter owns format detection, decode/parse rules, parser/index/source-region facts, ref creation/resolution, and operation algorithms. | Adapter-private values are usable only behind the creating adapter boundary and never become caller data or serialized facts. |
| Document acquisition and immutable bytes/text | **Open.** Candidates A, B, C, and E can leave acquisition inside the adapter boundary; candidate D may assign immutable acquisition/storage mechanics to a core primitive. | The gate must name the acquisition point, refresh allowance, retention bound, and consumers. A core byte view, if approved, is not adapter parser state. |
| Navigation configuration-source loading | Current navigation responsibility for project/user configuration input. | This responsibility is unrelated to document-file byte acquisition and does not decide candidate D. |
| Default UTF-8 full-read fallback | Navigation owns the fallback policy and result semantics. | The gate must define how it reads the approved document view without inspecting adapter-private decode/parse/index state. |
| Refs | The adapter generates and interprets refs; navigation and other callers validate only the shared non-empty-string boundary and pass refs unchanged. | A ref is interpreted against the owner-approved document view. Reuse adds no caller-visible state ID and no cross-invocation stability promise. |
| Protocol and output | Existing protocol/output owners keep all public shape and projection semantics. | Source, parser, handle, snapshot, and cleanup facts never enter protocol, readable output, continuation, logs, schemas, or examples. |
| Local service or future external host | No execution/storage ownership is granted by this change. A later capability may keep state host-local only after defining its own boundary. | No cross-request cache, serialized handle, public session ID, or unbounded interactive retention is permitted here. |

## Decisions

### Decision 1: The reusable outcome is an invocation-private selected document view, not a public session

**Status: target and boundary confirmed; concrete representation remains open.**

The accepted problem is duplicate complete preparation attributable solely to stages composed inside one navigation invocation. “Selected document view” is a conceptual outcome that may contain two separable layers: an immutable source byte/text view and adapter-private decoded, parsed, indexed, source-region, or ref-related facts. The gate may approve one or both layers and must name their separate storage and lifetime owners. The term does not imply that they share one type.

The reusable outcome:

- is bounded by one navigation invocation and cannot be looked up by a caller-visible ID;
- leaves all format interpretation and private prepared facts with the adapter;
- allows only the creating adapter to use the adapter-private portion;
- may allow navigation's default full-read fallback to consume a core-owned immutable source view only if candidate D or an explicit bounded combination is approved;
- cannot be serialized into `RequestEnvelope`, `ProtocolResponse`, readable output, continuation, ref, invocation log, or service status;
- does not make adapter operation results or refs valid beyond the approved source-view lifetime;
- releases each unsupported/invalid candidate's adapter-private portion before advancing under the approved bound, while a separately approved shared source view may remain reachable inside the same invocation; and
- releases every selected private portion and source view no later than the approved success, selection failure, operation failure, validation fallback, cancellation, or unwind-safe invocation endpoint.

“Invocation-private selected document view” is an outcome-level term, not the
name of a Rust type, a claim that one object owns both layers, or approval for a
`Session` trait.

### Decision 2: Navigation owns composition and lifecycle; adapters own state and algorithms

**Status: confirmed.**

Navigation must define the stage boundaries at which candidate state may be created, selected, made available to direct dispatch or a composition stage, and no longer reachable. The approved mechanism may keep the concrete creation/drop mechanics behind the adapter boundary, but it must still prove the navigation-owned lifetime bound. Navigation must not inspect or reinterpret adapter-private contents. The adapter owns format detection, decode/parse rules, parser/index structures, source-region mapping, ref creation and resolution, and operation algorithms over the approved view. Current navigation ownership of raw project/user configuration-source loading is not ownership of document-file acquisition; document source acquisition/storage remains an explicit gate decision.

The ownership split rules out:

- core-defined parser trees or generic resolved-node handles;
- navigation parsing refs to recover a node;
- adapter-owned auto-read eligibility or operation sequencing;
- a public or protocol-level state token;
- hidden cross-invocation caches used to simulate lifecycle reuse.

A core-owned source acquisition primitive remains a candidate only if the adapter still owns decode/parse and the total design also addresses repeated adapter parse where required. A private shared helper remains possible only for mechanics with the same lifecycle and failure semantics across real adapters.

### Decision 3: Resource success is proved per path, not by interface shape

**Status: confirmed; numeric budgets and approved snapshot are open.**

Instrumentation must distinguish at least source acquisition, decode, complete parse/model construction, and cleanup. Evidence must cover:

1. declared-adapter and automatic-discovery direct outline/read/find/info;
2. each unsupported/invalid automatic-discovery candidate and the eventually selected candidate;
3. cost threshold match, miss, measurement failure, content-hook failure, and default full-read fallback;
4. eligible unique-ref nested read success, adapter diagnostic, invalid nested result, and invalid composed response;
5. a path replaced or modified at controlled points between probe, policy, base operation, and nested read;
6. normal return, early return, error, cancellation if supported, and panic/unwind-safe destruction.

The target is not “one parse for the entire invocation” regardless of semantics. It is “no repeated complete preparation of the same approved document view solely because navigation composed internal stages.” An explicitly approved refresh or retry may prepare another view, but its trigger, error mapping, and bound must be normative and tested; routine fallback cannot silently reparse until it succeeds.

### Decision 4: Six candidates remain distinct until the owner gate

**Status: comparison recorded; no candidate selected.**

The candidates are evaluated against shared obligations rather than surface similarity:

| Candidate | Automatic discovery and unsupported cleanup | Direct operation | Full-read policy and fallback | Nested read | Snapshot / TOCTOU | Process and maintenance boundary |
| --- | --- | --- | --- | --- | --- | --- |
| **A. Probe returns an opaque prepared state companion** | Each probe attempt may return public evidence plus an internal state owned by that adapter; unsupported/invalid state must be destroyed before or while advancing under a bounded policy, and only selected state advances | Can reuse selected probe preparation if every operation accepts the opaque companion | Adapter hooks can reuse it; core default UTF-8 fallback cannot inspect it without an additional source seam | Same selected companion can reach ordinary read | Naturally favors selected probe view; refresh and JSON post-probe change behavior require explicit rules | Small selection delta in concept, but type erasure/downcast, ownership transfer, and fallible cleanup can become unsafe or adapter-framework complexity; the opaque value cannot cross a process |
| **B. Candidate-scoped open/probe handle** | Navigation opens one candidate handle, probes through it, drops failed handles, and promotes the selected handle; RAII can make cleanup explicit | Selected handle can dispatch ordinary operations without a second open | Cost, content, facts, structured fallback, and nested read can share one handle; default fallback needs a handle-owned source operation or an approved core snapshot | Ordinary read method on the selected handle can reuse state | Snapshot begins at candidate open unless separately refreshed | Strong lifecycle expression without downcast, but creates an object/lifetime abstraction before cheap rejection and can expand into a generic document object; in-process only unless a future adapter host contains the handle |
| **C. Operation-shaped invocation session** | If created only after selection it cannot reuse automatic-discovery probe work; if it includes probe it converges toward candidate handle A/B and must adopt their cleanup rules | Clean operation-shaped dispatch after session creation | Can offer the fixed operations and declared hooks over one state | Naturally supports repeated ordinary read | Snapshot normally begins at session creation; the relation to prior probe is unresolved | Familiar interface but broadest risk of a second adapter framework, method growth, and premature request-scoped state; external use would require host-local lifetime, not public session IDs |
| **D. Core-owned document acquisition/byte view plus adapter-owned decode/parse/ref/source-region behavior** | Core may acquire one immutable byte view and lend/share it across candidate probes; each adapter must still discard its private failed-candidate parse state | Selected adapter can prepare once from the shared source only if an additional private prepared-state boundary exists | Navigation's default fallback can reuse bytes; adapter cost/content can share adapter-private preparation if the combined design provides it | Nested read can reuse only with the same private preparation boundary | Makes source snapshot explicit and can preserve one inode/byte view; this deliberately changes path-reopen behavior | Moves document acquisition/storage and source lifetime into core, may allocate large shared buffers, and by itself removes neither repeated decode nor parse; external transport of bytes is a separate future design |
| **E. Adapter-local or composition-local reuse** | Existing selection may remain; failed candidates use Current cleanup | Can special-case only chosen adapter/path, often leaving probe duplication | Can optimize one adapter's hooks or one navigation branch | Can optimize auto-read locally | Snapshot and lifecycle risk becoming implicit or different per branch | Lowest initial surface, useful as evidence, but likely duplicates orchestration, requires hidden cache/thread-local/downcast or combination methods, and may not form a durable cross-adapter contract |
| **F. Current independent operations** | Current probes return public facts and leave no state; Rust locals clean up each attempt | Probe and operation prepare independently | Measurement/content/structured fallback prepare independently | Base and nested read prepare independently | Every stage may reopen the path; JSON uses a post-probe reload failure for invalid changed content | No interface change and simplest ownership, but it does not solve the accepted resource problem |

No row is approved merely because it appears smaller. Candidate C is not the default. Candidate D is incomplete unless it also proves required adapter-private parse reuse. Candidate E is acceptable only if owner evidence shows the reusable obligation is truly local rather than a shared lifecycle. Candidate F remains the compatibility baseline and rollback implementation, not a solution.

The architecture/product owner may approve a deliberately bounded composition of candidates (for example, a source primitive plus a typed adapter-private prepared boundary), but must name each part and reject the unused generality.

### Decision 5: Selection, full-read, and nested-read semantics remain observable compatibility constraints

**Status: confirmed constraints; source-view behavior open.**

- **Automatic discovery:** registry order, first-supported selection, public probe validation, and collected candidate failure evidence remain unchanged. Private state from an unsupported or invalid candidate cannot be passed to another adapter. Cleanup failure must not silently convert an unsupported candidate into supported; whether cleanup failure is reportable or only internal is part of the gate.
- **Direct operation:** the selected adapter still receives one closed operation-specific input and returns one typed result or adapter diagnostic. Reuse cannot add a generic parameter/state lookup to that input.
- **Full-read:** Current mode resolution, threshold comparison, measurement failure fallback to structured outline, content/facts hooks, and default UTF-8 fallback remain navigation-owned. The approved design must say which source view the default fallback reads and whether a threshold miss retains prepared state for structured outline.
- **Auto-read:** Current-result unique-ref eligibility, opaque ref pass-through, read page `1`, validated nested read, composed-response validation, and silent fallback to the validated base result remain unchanged. State reuse cannot turn nested read failure into a public partial status.
- **Refs:** refs are interpreted against the adapter's approved current view. Reuse must not make navigation parse them or promise cross-invocation stability.

### Decision 6: Public contracts remain unchanged and private state stays inside the executing process

**Status: confirmed.**

This change does not modify protocol, output, ref, CLI, parameter, schema, example, or continuation shape. Tests must assert absence of state IDs, handle metadata, snapshot metadata, cleanup fields, parser facts, and nested failure details from both protocol JSON and readable output.

This change's implementation scope is the static linked adapter path. A future external adapter host could keep an invocation object inside that host process, but this change does not define its transport or authorize serializing private state. Local core service mode may continue to reuse core-owned project/config/registry facts; it must not turn this invocation-local state into a cross-request cache. Service disabled/enabled paths must invoke the same approved adapter lifecycle if that change later adopts this handoff.

### Decision 7: Related changes receive handoffs, not implementation from this change

**Status: confirmed.**

- `interactive-outline-selection` composes outline and one or more later reads across user interaction. Its maintainer must decide whether one interactive workflow is one reuse invocation or multiple independent invocations; this change does not extend a private state lifetime across an unbounded prompt.
- `add-ast-grep-code-adapter` keeps ast-grep models private. It must not be forced onto an unapproved state interface; after approval it supplies lifetime and memory evidence appropriate to borrowed parser structures.
- `enable-local-core-adapter-service-mode` remains core-local and cannot use this change to claim a public adapter runtime or cross-invocation parser cache.
- The JSON adapter's Current post-probe reload diagnostic and mutation test are explicit migration inputs. The JSON owner must accept any replacement snapshot rule in its own owner materials; this change does not silently overwrite it.

These are handoff surfaces, not prerequisites that merge the changes or
authorize edits outside this directory. The proposal owns the independent
change boundary; this Decision owns only the technical handoff constraints
listed above.

## Risks / Trade-offs

- **[A “session” name predetermines an over-broad framework]** → keep requirements outcome-based, compare all six candidates, and require owner approval of exact methods, types, and lifecycle before code.
- **[Reuse silently changes which file version a ref addresses]** → approve a stage-by-stage snapshot matrix and deterministic JSON/Markdown mutation cases before normative mechanism refinement.
- **[Unsupported candidate state leaks during automatic discovery]** → require bounded RAII/drop evidence for every continue/error branch and forbid cross-adapter state transfer.
- **[Core source sharing erodes adapter ownership]** → core primitives, if approved, expose only immutable acquisition mechanics; decode, parse, indexes, refs, and algorithms remain adapter-owned.
- **[Fallback paths reintroduce duplicate work]** → instrument threshold match/miss/error, content/facts hooks, structured fallback, and default UTF-8 fallback separately.
- **[Private state becomes an accidental public or log contract]** → keep protocol/schema unchanged and add serialization/log non-leakage checks.
- **[Large parsed models live longer and increase peak memory]** → require peak/lifetime evidence and release state immediately after the last eligible stage; do not retain it across invocations or unbounded interactive waits.
- **[External/service aspirations force serializable handles]** → scope implementation to linked adapters and record only a future host-local lifetime requirement.
- **[Local special-casing becomes permanent duplication]** → accept local reuse only with evidence that no shared lifecycle exists; otherwise select a typed common boundary after two real adapter implementations.
- **[Concurrent file mutation makes cleanup or diagnostics nondeterministic]** → use controlled mutation barriers and specify whether the result is snapshot success, changed-document diagnostic, or another bounded owner-approved outcome.

## Migration Plan

1. Complete tasks 1.1–1.6 and present one decision packet containing the six-candidate matrix, baseline and target counts, the responsibility/lifetime map, stage-by-stage source-view semantics, automatic-discovery cleanup, full-read fallback, nested-read fallback, and process-boundary consequences.
2. Complete task 1.7 by obtaining explicit architecture/product-owner approval of the entire packet. Reviewers and agents may improve evidence but cannot select or mark a candidate approved.
3. Complete task 1.8: append a Decision naming the exact mechanism and rejected generality, close or answer every Open Question, and refine every mechanism-neutral delta with complete lifecycle, ownership, snapshot, error, cleanup, and fallback rules.
4. Complete task 1.9's cross-artifact audit. Owner-doc, test, and implementation work remains blocked until this audit passes.
5. Follow the testing owner and Case-maintenance workflow, prove the current tree closes, and establish current/failing count, mutation, cleanup, and non-leakage evidence.
6. Implement the smallest approved linked-adapter vertical slice at the exact owner boundaries named by the decision, then align Markdown. Change navigation or adapter-contract representation only where the approved candidate requires it; do not manufacture a shared data structure for a local candidate. While the JSON capability is not archived, treat JSON counts and TOCTOU behavior only as a gate/handoff and do not modify its normative or production owner from this change. Hand off compatible requirements to the later JSON owner, code adapter, interactive selection, and service mode through their own changes.
7. Validate direct operations, automatic discovery, every full-read branch, auto-read success/fallback, controlled TOCTOU, cleanup, protocol/readable non-leakage, workspace checks, and release-package behavior.

Rollback before publication removes the private reuse mechanism and returns to Current independent operation preparation; public payloads require no migration. If the approved implementation changes snapshot/TOCTOU behavior, rollback is a behavior change and must restore the corresponding owner docs and mutation evidence rather than being described as operationally invisible.

## Open Questions

All four gate areas below form one explicit **architecture/product-owner
decision**. A partial answer does not unlock implementation, and an agent cannot
close a gap by choosing a conventional default.

### Gate G1: Mechanism and responsibility split

1. Which candidate, or precisely bounded combination of candidates, is approved?
   What Rust ownership/type shape is allowed, and what tempting generality is
   explicitly rejected?
2. Which owner stores document bytes/text, which owner stores adapter-private
   preparation, and which component controls reachability? Must every direct
   operation reuse selected probe preparation when available, or are any
   format-evidence-only probe reads intentionally exempt?

### Gate G2: Source view, refs, and TOCTOU

3. Does the selected candidate's successful probe view become the
   base/full-read/nested-read view, or is there one deliberate refresh after
   selection? At exactly which stage is the source snapshot fixed?
4. For path replacement, in-place mutation, deletion, encoding change, and
   parser-invalid replacement between stages, should the invocation return a
   result from the captured view, a changed-document diagnostic, a normal owner
   diagnostic, or another bounded outcome?
5. How is the Current JSON `json-document-changed-after-probe` behavior
   migrated? Which JSON owner material and deterministic TOCTOU cases must
   change before implementation is accepted?

### Gate G3: Cleanup, fallback, and resource bounds

6. During automatic discovery, when exactly is unsupported/invalid candidate
   state destroyed, what resource bound applies while advancing, and can
   cleanup failure affect collected candidate evidence or the final selection
   diagnostic?
7. On full-read threshold match, miss, measurement error, content/facts error,
   and absent content hook, which prepared/source view is retained or released?
   How does the navigation-owned default UTF-8 fallback consume the approved
   view without inspecting adapter-private state?
8. On nested-read adapter failure, invalid nested result, invalid composed
   response, or cancellation, is Current silent base-response fallback
   preserved in every case, and when is private state released?
9. Are destructors/RAII sufficient for cleanup, or does any approved resource
   require fallible close/cancellation? If close fails, which owner receives
   the diagnostic without adding public cleanup fields?

### Gate G4: Execution boundary and handoffs

10. Is implementation explicitly limited to static linked adapters? What
    handoff statement is required so a future external adapter host or local
    service keeps state host-local and invocation-local without a public
    session ID?
11. Does `interactive-outline-selection` define each user-confirmed read as a
    new invocation, or may it retain state across a bounded prompt? What
    timeout/cancellation/memory bound is required if retention is approved
    there?
12. In what order do Markdown, the JSON owner, code adapter, interactive
    selection, and local service mode accept handoffs, and which owner can
    block archive of this change?

Gate completion must record the selected candidate parts and rejected
generality, the final responsibility/lifetime map, a stage-by-stage source-view
table, a cleanup/resource table, compatibility and handoff outcomes, and the
explicit approval source. Task 1.8 then turns those outputs into a numbered
Decision and complete delta requirements.

These answers are required inputs to task 1.7. `tasks.md` owns the remaining
unlock and apply sequence.
