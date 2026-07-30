本 delta spec 是 `replace-probe-traversal-with-inferred-routing` 的临时验证工件：它删除 probe schema/runtime validation pipeline，并使 routing diagnostic schema 与 owner-defined exact details 同步。

## MODIFIED Requirements

### Requirement: Runtime JSON validation uses typed field metadata where appropriate

Runtime validation MUST preserve owner semantics when it uses typed-field metadata for reusable field-level checks on protocol, manifest, and other current JSON surfaces. Semantic validation stays with the owning contract. Because probe result is removed from the protocol/adapter contract, contract-validation MUST delete its probe runtime validation entry point, probe decoder hook, consumer-local probe field-definition sets, nested reason validation, and schema-name registration rather than retain an unreachable validator.

#### Scenario: Manifest field type is invalid

- **WHEN** a manifest JSON field has the wrong type
- **THEN** typed-field validation can report the field failure
- **THEN** adapter-contract maps it into the appropriate boundary diagnostic

#### Scenario: Removed probe value is presented

- **WHEN** a caller or test presents a former probe-result JSON value after migration
- **THEN** no probe-specific runtime contract validator or decoder exists
- **THEN** contract-validation does not treat the removed value as a current public surface

### Requirement: Schema and examples sync with owner changes

Changes to machine-readable fields, examples, protocol shapes, diagnostic detail shapes, adapter metadata, output payloads, or config schemas MUST update the corresponding schema and example validation material in the same change. Removing probe in this change MUST delete its standalone schema, examples, fixtures, schema index entries, runtime validator, and validation tests. Discovery of a real owner-backed probe consumer MUST stop current apply and return to artifacts/human approval rather than weaken deletion with an implementation-time exception.

#### Scenario: Protocol field changes

- **WHEN** a protocol result field is renamed, added, or removed
- **THEN** protocol schema and examples are updated
- **THEN** validation catches stale examples

#### Scenario: Probe surface is removed

- **WHEN** the adapter/protocol owner removes probe result and probe candidate details
- **THEN** probe-result schema, examples, fixtures, runtime validator, typed-field consumer definitions, and conformance references are deleted in the same change
- **THEN** protocol-response schema/examples adopt the exact `FORMAT_UNKNOWN` and `FORMAT_AMBIGUOUS` details from diagnostics-contract
- **THEN** no inspection-only validation path is retained by this change; discovery of a real owner-backed consumer stops current apply and returns to artifacts/human approval
