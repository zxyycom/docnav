## ADDED Requirements

本文是 `simplify-typed-field-infrastructure` 的未审核临时 delta，仅描述归档后应保持的 capability contract。

### Requirement: Resolution semantics survive internal simplification

CLI/config resolution MUST preserve ordered source precedence、presence semantics、selected value validation、default fallback and provenance independently of its internal field representation or source adapter layout. Any intended change to those observable semantics MUST be declared by a separate capability delta.

#### Scenario: Internal representation changes

- **WHEN** an internal field helper、source adapter or package boundary is simplified
- **THEN** the same ordered sources produce the same selected value、fallback and provenance facts
- **THEN** existing consumer diagnostic mapping receives equivalent resolution facts

#### Scenario: Simplification would change observable resolution

- **WHEN** a proposed internal change would alter precedence、validation、fallback or provenance
- **THEN** the current capability contract rejects behavioral equivalence
- **THEN** the intended behavior requires an explicit capability delta before implementation
