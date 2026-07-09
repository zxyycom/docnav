本 spec delta 定义 `typed-fields` 的新增要求：`FieldDefSet` 必须提供可复用的 processing path metadata lookup 和 candidate JSON value validation；当前 config-source 所需的 compound JSON shape helper 只有在既有 owner-specific 数组配置校验无法满足 parity 时才作为最小扩展使用。

本 spec delta 只拥有 typed-field metadata lookup 和 candidate value validation helper。它不拥有 config source priority、CLI syntax、adapter-id namespace policy、public diagnostic codes 或 JSON schema generation。数组/对象 helper 只有在对应 owner 选择把当前 config-source subset 表达进 typed-fields 时才生效。

## ADDED Requirements

### Requirement: Definition sets expose processing-path validation metadata

Definition sets MUST expose a deterministic projection that lets consumers find field metadata by processing id and structured source path. The projection MUST include field identity, source path, value kind, constraints, nullability/presence, defaults when declared, and enough declaration metadata for the consumer to validate a candidate JSON source value for declared scalar or simple value fields. When a current config-source owner elects to express an array/object shape through typed-fields, the projection MUST also preserve the current subset's compound node kind, object member metadata, array item metadata, and nested source path facts. Duplicate or incompatible declarations for the same processing id and structured path MUST be rejected or surfaced as deterministic projection errors; consumers MUST NOT have to choose between competing field facts.

#### Scenario: Consumer finds config field metadata

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `defaults.pagination.limit`
- **THEN** the definition set returns the matching field metadata when that path is declared
- **THEN** the consumer can validate a candidate JSON value without reconstructing the field's value kind, constraints, or containing shape

#### Scenario: Consumer finds nested config-source metadata when declared

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `outline.mode_rules[].mode`
- **THEN** the definition set returns metadata for the nested field when the owner has declared that array item object shape through typed-fields
- **THEN** the consumer can report failures against the nested processing path

#### Scenario: Missing path is distinguishable

- **WHEN** a consumer asks a definition set for metadata at an undeclared config path
- **THEN** the definition set reports that no field is declared at that path
- **THEN** the consumer remains responsible for mapping that result into an unknown-field or unsupported-option diagnostic

#### Scenario: Duplicate path is rejected deterministically

- **WHEN** a definition set contains two incompatible declarations for processing id `config` and path `defaults.output`
- **THEN** the projection reports a duplicate or incompatible declaration error
- **THEN** the consumer does not silently choose one declaration for validation

### Requirement: Typed-field validation returns canonical values

Typed-field validation MUST allow consumers to validate a candidate JSON value for a declared scalar field and obtain the canonical typed value, or a validation failure that preserves field identity, processing path, value kind, and constraint reason. When a current config-source owner expresses a compound field through typed-fields, validation MUST support that declared subset and preserve nested path and shape reason in failures. Source-specific coercion and source priority MUST remain consumer-owned before typed-field validation is called.

#### Scenario: Candidate value is canonicalized

- **WHEN** a consumer validates the candidate JSON value `true` accepted by a declared boolean field
- **THEN** typed-field validation returns the canonical boolean typed value
- **THEN** the consumer can serialize the canonical value into its own inspection, config-source, or request representation

#### Scenario: Candidate value fails constraints

- **WHEN** a consumer validates a candidate JSON value that violates a declared numeric range
- **THEN** typed-field validation returns a failure with the field identity, processing path, received value kind, and range reason
- **THEN** the consumer maps the failure into its own diagnostic boundary

#### Scenario: Compound candidate fails nested shape when typed-fields owns that shape

- **WHEN** a consumer validates a candidate JSON array whose item object is missing a required declared member
- **AND** the owner has declared that compound config-source shape through typed-fields
- **THEN** typed-field validation returns a failure with the field identity, processing path, nested item/member path, received value kind, and shape reason
- **THEN** the consumer maps the failure into its own config-source diagnostic boundary

### Requirement: Processing-path projections preserve consumer ownership

Processing-path metadata projections MUST NOT own source priority, CLI flag syntax, config command shape, adapter selection, public diagnostic code identity, protocol envelope shape, output rendering, adapter-id namespace policy, or generic JSON schema policy. Those policies MUST remain with the consuming capability.

Owner-specific array/object validation MAY remain outside typed-fields when that owner can preserve source path, diagnostics, and navigation/config parity without duplicating field semantics in core CLI. Typed-fields MUST NOT be treated as a required replacement for existing outline array validation unless the implementation audit shows the current owner path cannot satisfy the config-source parity target.

#### Scenario: Consumer maps validation result

- **WHEN** typed-fields returns a validation failure for a config processing path
- **THEN** typed-fields provides field-level failure facts
- **THEN** the consuming config or navigation owner chooses the public diagnostic code, source priority behavior, source-specific coercion, and output projection
