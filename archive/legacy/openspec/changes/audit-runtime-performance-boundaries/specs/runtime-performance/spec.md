**本文是 `runtime-performance` 新 capability 的临时 delta spec：它定义未来审计必须满足的证据与决策边界，不表示 Current 性能、已批准 budget、gate 或优化实现。**

## ADDED Requirements

### Requirement: Runtime performance audit covers the full owned boundary

Runtime performance auditing MUST separately cover startup time, end-to-end wall time and CPU, input I/O and repeated preparation, peak and retained memory, output size and pagination, package size, and scaling under extreme inputs. The audit MUST classify each workload as `representative` or `stress/adversarial`; evidence from either class MUST NOT be presented as proof for the other class.

#### Scenario: Representative workload is reported

- **WHEN** an audit measures a workload intended to represent normal Docnav use
- **THEN** the record identifies the workload as `representative` and states the format, scale, structure, operation, output mode, and pagination shape that make it representative
- **THEN** the report does not claim that this workload establishes behavior for extreme inputs

#### Scenario: Extreme input scaling is investigated

- **WHEN** an audit investigates very large node counts, long keys or refs, deep or wide structures, high or zero match counts, later pages, full-root reads, repeated calls, or another adversarial shape
- **THEN** the record identifies the workload as `stress/adversarial`
- **THEN** the audit compares explicit input-size or structure steps and reports the applicable time, CPU, I/O, memory, output, pagination, and package dimensions without treating the result as normal-use baseline evidence

### Requirement: Initial audit workload is bounded before measurement

Before any measurement, the audit MUST define a finite initial workload packet and stopping rule. The packet MUST use stratified required cells rather than a Cartesian product: one startup and one release package cell; outline, find, and ref-derived read cells for one primary Current format with `protocol-json` and `readable-view` distributed across those cells; one first-page and one later-page cell; one outline cell for exactly one secondary Current format when available; and stress cells consisting of a three-tier outline scale ladder on one Current format, find miss and root read at the largest tier, one long key/label/ref under a small limit, and one retained-memory lifecycle cell only when a Current same-process surface makes it meaningful. Unselected combinations MUST be labeled `unmeasured/future`; unavailable and not-applicable required cells MUST cite their evidence.

#### Scenario: Finite packet is fixed

- **WHEN** task 2 is ready to begin runtime measurement
- **THEN** `audit-report.md` already names every required cell, selected binary/format/operation/output/page/stress shape, fixture selection rule, and the unavailable or not-applicable rules
- **THEN** no measurement begins by enumerating every format, operation, output, page, limit, query, ref, and stress combination

#### Scenario: Initial audit reaches its stopping rule

- **WHEN** every required cell has either a complete measurement record or an evidence-backed unavailable/not-applicable disposition, the three scale tiers are reported, and all remaining combinations are labeled `unmeasured/future`
- **THEN** the initial measurement set stops expanding even if attribution remains `unattributed`
- **THEN** an additional workload requires a report-backed decision-critical gap and separate approval rather than automatic matrix expansion

### Requirement: Comparable measurements are self-contained

Every measurement used for comparison MUST record binary identity, full command and flags, fixture identity and relevant structure, output mode, page, limit, query and ref when applicable, build/source/dependency identity, process boundary, host/runtime/storage context, cache state, warmup and repeats, measurement tool and metric definitions, raw samples, aggregation method, stdout/stderr size, continuation outcome, and all applicable timing, CPU, I/O, preparation-count, peak-memory, retained-memory and package-size results. Unknown and not-applicable fields MUST be explicit.

#### Scenario: A measurement becomes eligible for baseline comparison

- **WHEN** maintainers propose comparing a measurement with a later run
- **THEN** both records identify the same command semantics, fixture, output mode, page/limit/ref/query, build profile, process boundary, host assumptions, cache state, repeat method, metric definitions, and output handling
- **THEN** any material difference is reported before a before/after claim is made

#### Scenario: Retained memory is not meaningful for a short-lived process

- **WHEN** a one-shot CLI process exits before an in-process quiescent retention point can be observed
- **THEN** retained memory is recorded as not-applicable rather than inferred from peak RSS
- **THEN** a repeated in-process or service workload records its lifecycle and quiescent sampling point before making a retention claim

### Requirement: Evidence states remain distinct

The audit MUST distinguish `seed observation`, `observation`, `reproducible baseline`, `approved budget`, and `approved gate`. Incomplete historical or worktree measurements MUST remain seed observations until reproduced with the required metadata, and no OpenSpec artifact alone MUST be treated as Current implementation or release evidence.

#### Scenario: Historical measurement lacks reproducibility metadata

- **WHEN** a timing, RSS, output-size, package-size, or tokenizer measurement lacks required command, build, host, cache, repeats, dependency, or metric-definition context
- **THEN** the audit retains it only as a seed observation with the missing fields identified
- **THEN** the value is not used as a Current baseline, budget, gate, or optimization-benefit claim

#### Scenario: Worktree evidence uses an unapproved dependency

- **WHEN** a measurement depends on a worktree-only or unapproved dependency state
- **THEN** the record identifies that dependency state
- **THEN** the result is not labeled as a Current binary or release baseline

### Requirement: Attribution precedes optimization selection

For each material observation, the audit MUST attribute supported cost to one or more of `startup-process`, `input-io`, `probe-routing`, `decode-parse-index`, `operation-traversal-lookup-search`, `repeated-preparation-composition`, `cost-calculation`, `pagination-output-construction`, `serialization-write`, `memory-retention`, `package-dependency`, or `unattributed`. Attribution MUST cite comparison, profiling, instrumentation, counts, or exclusion evidence and MUST preserve uncertainty; a total time, RSS value, or output byte count alone MUST NOT select an optimization.

#### Scenario: Total runtime is observed without internal evidence

- **WHEN** an end-to-end command is slow but no comparison, profile, instrumentation, count, or exclusion evidence isolates the cost
- **THEN** the report uses `unattributed` for the unresolved portion
- **THEN** it requests the smallest next measurement rather than naming a parser, cache, state-reuse, pagination, allocator, or dependency fix

#### Scenario: More than one category contributes

- **WHEN** evidence shows that startup, parsing, repeated preparation, output construction, serialization, or another category each contributes materially
- **THEN** the report preserves the separate contributions or marks multi-category attribution
- **THEN** it does not force a single-owner explanation unsupported by the evidence

### Requirement: Observations are non-blocking until human approval

Seed observations, observations, and reproducible baselines MUST be non-blocking by default. A numeric budget MUST require explicit human approval of the workload, metric, value or range, statistical rule, build, host, cache state, noise tolerance, and review conditions. A blocking gate MUST require a separate explicit human approval of enforcement owner, execution surface, failure semantics, and update or removal procedure.

#### Scenario: A baseline is recorded without a budget decision

- **WHEN** repeatable measurements establish a reproducible baseline
- **THEN** normal product verification, merge, and release remain unblocked
- **THEN** no threshold or regression tolerance is inferred from the baseline

#### Scenario: A maintainer proposes a blocking gate

- **WHEN** a maintainer wants performance evidence to fail CI, merge, or release
- **THEN** the decision packet includes the exact workload and measurement contract plus enforcement ownership and failure semantics
- **THEN** the gate remains absent until a human explicitly approves those terms

### Requirement: Performance findings return to the behavior owner

The audit MUST hand each proposed fix to the capability or change that owns the affected behavior. It MUST remain independent of the Current `json-adapter` owner, `redesign-token-cost-estimation`, `reuse-adapter-document-state`, and `redesign-find-result-model`; none of those owners or changes MUST depend on this audit, and this audit MUST NOT reopen the archived `add-json-adapter` change or implement/duplicate their JSON behavior, estimator, reusable-state mechanism, find model, or work-budget decisions.

#### Scenario: Attribution identifies an owner-specific path

- **WHEN** evidence attributes a problem to adapter parsing/ref/search, core routing/process, token cost, repeated preparation, find semantics, output/pagination, or package dependencies
- **THEN** the report identifies the corresponding adapter, core/navigation, token-cost, document-state, find, protocol/output, or release/dependency owner
- **THEN** implementation is proposed in that owner change or a new owner-specific change rather than in the runtime-performance audit

#### Scenario: Static quality reporting later links to runtime evidence

- **WHEN** a future integration presents repository quality snapshots beside runtime performance results
- **THEN** `repository-quality-observability` continues to own static code-quality snapshots
- **THEN** `runtime-performance` continues to own workload measurements, attribution, and performance decisions

### Requirement: Stable runtime performance guidance has a dedicated docs owner

After this capability is approved for apply, `docs/runtime-performance.md` MUST own the stable runtime performance workload, measurement, evidence-state, attribution, baseline/budget/gate, and owner-handoff rules. `docs/navigation.md` MUST direct readers to that owner when establishing or interpreting runtime performance baselines, budgets, audits, or optimizations. Existing tooling and repository-quality owners MUST retain their tool-execution and static-snapshot responsibilities and MUST NOT become owners of product runtime performance.

#### Scenario: Runtime performance owner docs are applied

- **WHEN** the approved capability is synchronized into long-term docs
- **THEN** the apply work creates or updates `docs/runtime-performance.md` with the approved stable content
- **THEN** it updates both the reading-path and rule-owner mappings in `docs/navigation.md` and validates those Markdown navigation paths

#### Scenario: Tooling or quality evidence is reused

- **WHEN** a runtime audit invokes an existing tool or presents a static quality snapshot beside runtime measurements
- **THEN** tooling continues to own how the tool is run and repository quality continues to own the static snapshot
- **THEN** `docs/runtime-performance.md` owns the runtime workload meaning, comparison, budget decision, and optimization handoff

### Requirement: Audit and human decisions gate optimization work

The change MUST create a purpose-named change-local `audit-report.md` before recording its blocking artifact audit, MUST complete that audit before defining or measuring the initial workload packet, MUST complete measurement and attribution reporting before workload or budget approval, and MUST obtain the required human approvals before any optimization task is created or implemented under this effort. Artifact completion MUST NOT itself authorize a benchmark framework, dashboard, cache, common producer/sink, public CLI/protocol change, CI gate, or owner-specific optimization.

#### Scenario: Planning artifacts are complete but unaudited

- **WHEN** proposal, design, delta spec, and tasks are present
- **THEN** task 1 first creates `audit-report.md` for artifact-audit and later runtime-performance evidence, then records the artifact-audit conclusion in that report
- **THEN** the change remains blocked until that recorded audit passes
- **THEN** no measurement, budget, gate, or optimization is represented as already approved

#### Scenario: Attribution report is ready for human decision

- **WHEN** representative and stress/adversarial records, scaling evidence, attribution, unknowns, and owner handoffs are complete
- **THEN** a human may approve selected workloads as baselines and may separately approve budgets or gates
- **THEN** optimization tasks remain conditional on those explicit decisions and are implemented only by the affected owner
