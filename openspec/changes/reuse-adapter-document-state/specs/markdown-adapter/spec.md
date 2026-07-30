**Interpretation:** This mechanism-neutral Target delta requires Markdown to
avoid duplicate complete preparation over the owner-approved invocation view,
but it does not select a prepared-state, handle, session, shared source, or Rust
type. `proposal.md` owns the change status; `design.md` leaves “approved
invocation lifecycle” and “approved document view” open; tasks 1.7–1.8 must
approve and define them before applying this delta.

## MODIFIED Requirements

### Requirement: Markdown adapter provides v0 document operations

The Markdown adapter MUST implement outline, read, find, and info for Markdown documents through the linked adapter contract. When navigation performs selection and one or more eligible Markdown stages over the same approved document view in one invocation, Markdown MUST participate in the approved lifecycle so the compatible source view and adapter-private decoded line, heading, and ref facts can prevent reacquiring, decoding, or parsing the complete view solely because navigation dispatched another stage. This requirement does not decide whether immutable document bytes are stored by Markdown or by an approved core acquisition primitive.

#### Scenario: Supported Markdown document

- **WHEN** the selected adapter is Markdown and the document is supported
- **THEN** outline, read, find, and info are available through the standard document operation flow

#### Scenario: Direct Markdown operation follows probe

- **WHEN** successful Markdown selection and a direct outline, read, find, or info dispatch use the same approved document view
- **THEN** the approved lifecycle makes the compatible source view and Markdown-private decoded/prepared facts available as defined by the owner-approved count policy
- **THEN** Markdown does not repeat complete preparation of that view solely because dispatch follows selection
- **THEN** the operation result and diagnostic contract remain unchanged

#### Scenario: Markdown base operation is followed by nested read

- **WHEN** navigation invokes Markdown read after an eligible outline/find result in the same approved document view
- **THEN** Markdown resolves the opaque ref against compatible private facts from that view
- **THEN** it does not reload or reparse that complete view solely because read is a nested operation
- **THEN** existing Markdown read content, cost, pagination, and error semantics remain unchanged

### Requirement: Probe recognizes only Markdown format support

Markdown probe behavior MUST identify Markdown support and report unsupported input without claiming non-Markdown format ownership. Public `ProbeResult` facts MUST remain unchanged by private lifecycle reuse. If the approved invocation lifecycle preserves a compatible source view and Markdown-private decoded preparation from a successful probe, later eligible stages over the same document view MUST reuse those facts according to the approved count policy without exposing them as probe metadata.

#### Scenario: Markdown file

- **WHEN** probe receives a Markdown document path
- **THEN** it reports supported Markdown facts

#### Scenario: Non-Markdown file

- **WHEN** probe receives a document that is not recognized as Markdown
- **THEN** it reports unsupported without parsing it as Markdown
- **THEN** any candidate-private preparation is released under the approved discovery cleanup policy

#### Scenario: Successful probe advances to operation

- **WHEN** Markdown probe selects a document view and navigation later dispatches an eligible operation over that view
- **THEN** the public probe evidence contains only the existing format support facts
- **THEN** the compatible source view and Markdown-private decoded facts may advance through the approved lifecycle
- **THEN** no parser model, state handle, or snapshot identifier appears in public probe output

### Requirement: Markdown supports declared unstructured full-read outline

Markdown unstructured full-read outline support MUST be declared through adapter hook metadata before navigation can use it. Normal structured outline behavior MUST remain unchanged when the policy does not apply. Cost measurement, full-content production, and structured-outline fallback over the same approved document view MUST use the compatible source view and Markdown-private parsed facts and MUST NOT each reload or reparse the complete view solely because navigation invokes separate hooks.

#### Scenario: Policy triggers unstructured full read

- **WHEN** navigation pre-dispatch selects unstructured full-read for a Markdown document
- **THEN** Markdown supplies the full content through the declared hook
- **THEN** the result is not represented as heading entries
- **THEN** cost and content stages reuse compatible Markdown preparation for the approved view

#### Scenario: Policy does not trigger

- **WHEN** unstructured full-read policy does not apply
- **THEN** Markdown uses normal structured outline behavior
- **THEN** compatible preparation produced while evaluating the policy remains reusable by normal outline until the approved lifecycle ends

#### Scenario: Full-read stage fails

- **WHEN** Markdown cost or content evaluation returns an adapter diagnostic
- **THEN** navigation observes the existing full-read fallback or diagnostic semantics
- **THEN** Markdown-private state is released under the approved failure cleanup policy
- **THEN** no private cleanup or state fact is added to the public result
