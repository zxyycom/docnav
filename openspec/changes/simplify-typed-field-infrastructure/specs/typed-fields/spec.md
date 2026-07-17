## ADDED Requirements

本文是 `simplify-typed-field-infrastructure` 的未审核临时 delta，仅描述归档后应保持的 capability contract。

### Requirement: Typed fields expose shared field semantics

Typed-fields MUST provide stable field identity、value/constraint validation、processing projection and attributed failure facts required by current production consumers. Source-specific policy and owner-specific application behavior MUST remain with their consuming capability. A surface without a production consumer or current owner contract MUST NOT be required by this capability.

#### Scenario: Multiple owners consume shared field facts

- **WHEN** protocol、navigation or adapter contracts consume the same field identity、validation or projection facts
- **THEN** typed-fields provides those shared facts through one reusable contract
- **THEN** each consumer retains its public mapping and domain behavior

#### Scenario: Behavior belongs to one owner

- **WHEN** a processing or materialization behavior is specific to one production owner
- **THEN** that owner implements the behavior at its boundary
- **THEN** typed-fields is not required to expose a generic form without another current shared consumer
