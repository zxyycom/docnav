## ADDED Requirements

### Requirement: Typed JSON contract validation preserves schema-backed contract behavior
Typed JSON contract validation MUST use typed field metadata and semantic validation for runtime decoding while preserving public JSON Schema files as contract material and CI validation inputs.

#### Scenario: Non-standard-parameter JSON does not inherit parameter source semantics
- **WHEN** manifest, probe, protocol request, or protocol response JSON is decoded through typed JSON contract validation
- **THEN** field-level constraints can reuse typed field metadata
- **THEN** the decoder does not apply standard parameter CLI, config, default source merge, or passthrough semantics
