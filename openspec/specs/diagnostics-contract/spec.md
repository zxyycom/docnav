# diagnostics-contract Specification

## Purpose
Define Docnav's stable diagnostic model: diagnostic code identity, canonical details, diagnostic record construction, owner/source attribution, primary failure projection, and validation material updates. CLI exit codes, raw protocol envelopes, and readable rendering consume this capability without redefining diagnostic identity.

## Requirements
### Requirement: DiagnosticCode owns identity and canonical details
Every stable diagnostic code MUST have a single identity owner and canonical detail shape. Other layers can add context only when the context preserves the code identity and detail semantics.

#### Scenario: Code appears at multiple surfaces
- **WHEN** the same diagnostic is projected to protocol-json and readable-json
- **THEN** both projections use the same diagnostic code
- **THEN** canonical details keep the same structured meaning

### Requirement: Boundaries produce DiagnosticRecords
Public boundaries MUST convert failures into diagnostic records with owner, source, message, and canonical details before protocol or readable output projection.

#### Scenario: Invalid caller input
- **WHEN** caller input violates a strict input boundary
- **THEN** the owning boundary constructs a diagnostic record
- **THEN** downstream output surfaces consume that record

### Requirement: Public failures expose one primary diagnostic
Failure outputs MUST expose a single primary diagnostic record for the failed operation. Additional context must remain secondary, stable, and subordinate to the primary cause.

#### Scenario: Multiple candidate adapter failures
- **WHEN** adapter selection observes several failed candidates
- **THEN** the operation failure has one primary diagnostic
- **THEN** candidate evidence is nested as stable details or secondary context

### Requirement: Legacy diagnostic sources do not define new contracts
Legacy error strings, ad hoc details maps, and removed diagnostic rule files MUST be treated as migration inputs and normalized into typed diagnostic records before public projection.

#### Scenario: Migrating a legacy error
- **WHEN** implementation still has a legacy failure source
- **THEN** the boundary maps it to a diagnostic record
- **THEN** public output exposes the diagnostic record contract

### Requirement: Diagnostic changes update validation materials
Changes to diagnostic code, detail shape, or projection MUST update the relevant schema, examples, fixtures, tests, or conformance materials owned by the affected surface.

#### Scenario: Detail shape changes
- **WHEN** a diagnostic detail field is added or removed
- **THEN** the owning validation material is updated in the same change
- **THEN** automated validation can detect drift
