**Interpretation:** This mechanism-neutral Target delta requires selection,
direct dispatch, full-read policy/fallback, and nested read to participate in a
bounded invocation lifecycle. It does not select a lifecycle representation,
document acquisition owner, or source snapshot. `proposal.md` owns the change
status; `design.md` leaves “approved invocation lifecycle” and “approved
document view” open; tasks 1.7–1.8 must approve and define them before applying
this delta.

## MODIFIED Requirements

### Requirement: Navigation selects adapter before adapter parameter extraction

Navigation MUST select the adapter using routing inputs and registry facts before filtering adapter-scoped entries for selected-operation candidate extraction and resolution. Full catalog config validation is a separate projection and MUST NOT be treated as adapter parameter extraction. The selected registry entry MUST expose an adapter definition for capability and linked strategy facts. Document-operation parameter declarations MUST come from the core catalog rather than that definition. During declared or automatic selection, navigation MUST bound the lifetime of any candidate-private state produced by the approved mechanism, release unsupported/invalid candidate state according to the approved cleanup policy, and allow only the selected adapter's state to advance into request dispatch. This requirement does not prescribe how that private state is represented.

#### Scenario: Multiple adapters exist

- **WHEN** registry contains multiple candidate adapters
- **THEN** navigation selects the adapter according to selection rules
- **THEN** only core catalog entries applicable to the selected adapter and operation participate in resolution
- **THEN** entries scoped to unselected adapters remain outside the operation field set

#### Scenario: Selected definition provides capability facts

- **WHEN** navigation has selected an adapter
- **THEN** it reads optional capability declarations and the linked strategy from the selected adapter definition
- **THEN** it reads parameter facts from the core catalog
- **THEN** any approved candidate-private state remains associated with the selected adapter without becoming a parameter fact

#### Scenario: Automatic discovery advances past a candidate

- **WHEN** an automatic-discovery candidate returns unsupported or invalid probe evidence
- **THEN** navigation preserves the existing candidate evidence and registry traversal semantics
- **THEN** candidate-private state is not passed to the next adapter
- **THEN** candidate-private state is released within the approved resource bound

#### Scenario: Selected candidate advances to dispatch

- **WHEN** a declared or automatically discovered candidate is selected
- **THEN** navigation may retain only that adapter's invocation-private document view for eligible later stages
- **THEN** the retained state remains opaque to navigation and absent from parameter resolution

### Requirement: Adapter-scoped cost threshold can trigger unstructured full-read outline

Navigation MUST run the unstructured full-read pre-dispatch check before normal adapter outline whenever the effective policy, selected adapter full-read capability declaration, and cost threshold all permit that result. Navigation MUST treat support, content, cost measurement, and result facts as parts of the selected adapter's declared full-read capability group. Cost measurement, selected full-read hooks, normal-outline fallback, and navigation-owned default UTF-8 fallback that operate on the same approved document view MUST participate in the same bounded invocation lifecycle and MUST NOT cause repeated complete preparation of that view solely because policy selected another stage.

#### Scenario: Threshold permits full read

- **WHEN** the selected adapter declares full-read capability support
- **AND** navigation determines the full read cost is below the effective threshold
- **THEN** navigation returns the declared unstructured outline result
- **THEN** normal structured outline dispatch is skipped for that request
- **THEN** cost and content/facts stages reuse compatible selected-adapter preparation for the approved document view

#### Scenario: Threshold selects normal outline

- **WHEN** the cost threshold is exceeded or support is undeclared
- **THEN** navigation dispatches normal adapter outline
- **THEN** compatible selected-adapter preparation for the approved document view remains available to that dispatch until no later eligible stage needs it

#### Scenario: Full-read capability is partially unsupported

- **WHEN** pre-dispatch policy requires a full-read capability fact outside the selected adapter definition
- **THEN** navigation follows the documented fallback or reports the unsupported boundary according to the owning policy
- **THEN** navigation bases that decision on the selected adapter definition's full-read capability group

#### Scenario: Full-read hook fails

- **WHEN** cost measurement, content, or result-facts evaluation fails
- **THEN** navigation preserves the existing owner-defined diagnostic or fallback semantics
- **THEN** it releases or retains private state only as required by the approved lifecycle
- **THEN** failure handling does not silently reparse the same view without an approved bounded retry rule

#### Scenario: Navigation uses default UTF-8 fallback

- **WHEN** the selected full-read path uses the navigation-owned default content fallback
- **THEN** the fallback reads the same approved source view selected for that invocation
- **THEN** navigation does not inspect adapter-private parser, index, or ref state
- **THEN** the fallback returns only the existing unstructured full-read facts

### Requirement: Navigation dispatches linked adapter handlers

After successful input resolution, standard type materialization, and configured core pre-dispatch checks, navigation MUST dispatch the closed standard operation input to the selected linked adapter strategy and return structured result or diagnostic facts to the owning output/protocol layer. The strategy reference and capability context MUST come from the selected adapter definition; applicable operation-specific typed fields or accessors MUST be built from core-catalog resolution. Any approved reusable preparation MUST remain associated with that same selected adapter under the approved private lifecycle, without prescribing its Rust representation. For a direct operation over the same approved document view used during successful selection, navigation MUST make compatible selected-adapter preparation available to dispatch so selection plus dispatch does not repeat complete preparation solely because they are separate stages. The selected strategy MUST NOT require a second caller-data argument or generic parameter handoff. It MAY return semantic validation diagnostics for conditions not guaranteed by core or MAY repeat a core check defensively.

#### Scenario: Dispatch succeeds

- **WHEN** navigation has constructed standard typed operation input
- **THEN** it calls the selected adapter strategy
- **THEN** it preserves the returned structured result facts for projection

#### Scenario: Direct dispatch reuses selected preparation

- **WHEN** successful selection and direct operation dispatch use the same approved document view
- **THEN** navigation carries the selected adapter's private lifecycle into dispatch
- **THEN** the adapter reuses compatible acquisition, decode, parse, or index facts rather than rebuilding the complete view solely for dispatch
- **THEN** the standard operation input and public result remain unchanged

#### Scenario: Dispatch returns adapter semantic diagnostic

- **WHEN** standard input is well-typed but violates a selected strategy precondition
- **THEN** the strategy returns a diagnostic before running the unsafe or invalid algorithm path
- **THEN** navigation preserves that diagnostic for normal protocol/readable projection
- **THEN** invocation-private state is released under the approved failure cleanup policy

#### Scenario: Dispatch combines separate core facts

- **WHEN** navigation dispatches a selected operation
- **THEN** the strategy implementation and private document semantics come from the selected adapter definition
- **THEN** adapter-scoped typed values come from entries applicable to that adapter and operation in core catalog
- **THEN** routing/strategy facts, private document state, and parameter facts remain owned by their separate sources

### Requirement: nested read reuses the selected document context

For an eligible unique ref, navigation MUST invoke the existing selected adapter read strategy without recursively invoking the CLI, selecting another adapter or executing an intermediate output plan. The nested read MUST use the same normalized document path, pass the ref unchanged and start at read page `1`; its remaining input MUST follow the existing read contract. When the base operation and nested read address the same approved invocation document view, navigation MUST retain the selected adapter's compatible private preparation through nested read and MUST release it after composition succeeds or falls back.

#### Scenario: ref remains opaque across composition

- **WHEN** navigation constructs the nested read input
- **THEN** it passes the candidate ref unchanged
- **AND** only the selected adapter read strategy parses the ref

#### Scenario: existing read inputs remain authoritative

- **WHEN** unique-ref orchestration invokes read
- **THEN** the nested read starts at page `1`
- **AND** uses the already resolved common input that applies to the existing read strategy
- **AND** any nested `ReadResult.page` retains its existing continuation meaning

#### Scenario: base and nested read share private preparation

- **WHEN** validated outline/find result eligibility triggers nested read over the same approved document view
- **THEN** navigation keeps the selected adapter lifecycle alive through read
- **AND** the adapter reuses the compatible source view and adapter-private parser/index/source-region/ref facts from the base operation
- **AND** no reusable-state identifier enters the ref, read input, or composed response

#### Scenario: validated read success is composed

- **WHEN** the selected adapter read returns a validated success
- **THEN** navigation constructs `auto_read` with reason `unique_ref` and the complete existing `ReadResult`
- **AND** validates the composed response before returning it to output orchestration
- **AND** releases invocation-private state after the final validation

#### Scenario: non-successful read keeps the base response

- **WHEN** the selected adapter read does not produce a validated success
- **THEN** navigation returns the validated base response unchanged
- **AND** does not add an auto-read status, reason or error object
- **AND** releases invocation-private state under the approved failure policy

#### Scenario: invalid composition keeps the base response

- **WHEN** nested read succeeds
- **AND** the candidate composed response does not pass protocol validation
- **THEN** navigation discards the candidate composition
- **AND** returns the already validated base response unchanged
- **AND** releases invocation-private state under the approved failure policy
