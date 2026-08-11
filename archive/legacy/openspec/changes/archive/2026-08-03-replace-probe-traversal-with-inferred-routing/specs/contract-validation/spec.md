本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `contract-validation` 尚未应用的 Target：验证 manifest exact-filename hints，删除 probe schema/runtime validation pipeline，并同步获批 routing diagnostics；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: Runtime JSON validation uses typed field metadata where appropriate

Runtime validation MUST preserve owner semantics when it uses typed-field metadata for reusable field-level checks on protocol, manifest, and other current JSON surfaces. Semantic validation stays with the owning contract. Because probe result is removed from the protocol/adapter contract, contract-validation MUST delete its probe runtime validation entry point, probe decoder hook, consumer-local probe field-definition sets, nested reason validation, and schema-name registration rather than retain an unreachable validator.

#### Scenario: Manifest field type is invalid

- **WHEN** a manifest JSON field has the wrong type
- **THEN** typed-field validation can report the field failure
- **THEN** adapter-contract maps it into the appropriate boundary diagnostic

#### Scenario: Manifest filename hint is invalid

- **WHEN** `formats[].filenames[]` contains an empty value, a path separator, `.` or `..`
- **THEN** manifest runtime validation rejects the descriptor
- **THEN** no routing index is constructed from the invalid hint

#### Scenario: Manifest suffix hint is invalid

- **WHEN** `formats[].extensions[]` contains an empty value, lacks a leading dot, consists only of `.`, or contains a path separator
- **THEN** manifest runtime validation rejects the descriptor
- **THEN** no normalized-suffix lookup view is constructed from the invalid hint

#### Scenario: Manifest compound suffix is valid

- **WHEN** `formats[].extensions[]` contains `.schema.json`
- **THEN** manifest runtime validation accepts the dotted compound suffix
- **THEN** schema and runtime validation preserve it for complete-basename longest-suffix matching

#### Scenario: Removed probe value is presented

- **WHEN** a caller or test presents a former probe-result JSON value after migration
- **THEN** no probe-specific runtime contract validator or decoder exists
- **THEN** contract-validation does not treat the removed value as a current public surface

### Requirement: Schema and examples sync with owner changes

Changes to machine-readable fields, examples, protocol shapes, diagnostic detail shapes, adapter metadata, output payloads, or config schemas MUST update the corresponding schema and example validation material in the same change. Removing probe in this breaking change MUST delete its standalone schema, examples, fixtures, schema index entries, runtime validator, and validation tests. Every discovered probe consumer MUST be deleted, migrated, or recorded as an explicit breaking impact in the owning material; discovery MUST NOT retain a compatibility validator or inspection-only path.

#### Scenario: Protocol field changes

- **WHEN** a protocol result field is renamed, added, or removed
- **THEN** protocol schema and examples are updated
- **THEN** validation catches stale examples

#### Scenario: Probe surface is removed

- **WHEN** the adapter/protocol owner removes probe result and probe candidate details
- **THEN** probe-result schema, examples, fixtures, runtime validator, typed-field consumer definitions, and conformance references are deleted in the same change
- **THEN** manifest schema/examples add `formats[].filenames[]` and express `formats[].extensions[]` as leading-dot basename suffixes that may contain multiple dots
- **THEN** protocol-response schema/examples adopt the exact `FORMAT_UNKNOWN`, registry-conflict, and selected `DOCUMENT_CONTENT_INVALID` details from diagnostics-contract
- **THEN** routing-only `NO_SUPPORTED_ADAPTER`, `FORMAT_MATCH`, and `FORMAT_AMBIGUOUS` candidate examples are removed when task 0.5 confirms they have no other Current owner
- **THEN** no compatibility validator or inspection-only path is retained by this change
- **THEN** every discovered consumer is deleted, migrated, or recorded as an explicit breaking impact
