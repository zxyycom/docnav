本 spec delta 定义 `typed-fields` 的新增要求：`FieldDefSet` 必须提供可复用的 config processing path metadata lookup、candidate JSON value validation 和 compound JSON shape validation projection，供 consumer 统一配置校验。

当前 change 只在 `openspec/changes/replace-config-commands-with-inspect/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Definition sets expose processing-path validation metadata

Definition sets MUST expose a deterministic projection that lets consumers find field metadata by processing id and structured source path. The projection MUST include field identity, source path, value kind, constraints, nullability/presence, defaults when declared, compound node kind when declared, object member metadata, array item metadata, and enough declaration metadata for the consumer to validate a candidate JSON source value or compound source value.

#### Scenario: Consumer finds config field metadata

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `defaults.pagination.limit`
- **THEN** the definition set returns the matching field metadata when that path is declared
- **THEN** the consumer can validate a candidate JSON value without reconstructing the field's value kind, constraints, or containing shape

#### Scenario: Consumer finds nested config metadata

- **WHEN** a consumer asks a definition set for metadata at processing id `config` and path `outline.mode_rules[].mode`
- **THEN** the definition set returns metadata for the nested field when the array item object shape declares that member
- **THEN** the consumer can report failures against the nested processing path

#### Scenario: Missing path is distinguishable

- **WHEN** a consumer asks a definition set for metadata at an undeclared config path
- **THEN** the definition set reports that no field is declared at that path
- **THEN** the consumer remains responsible for mapping that result into an unknown-field or unsupported-option diagnostic

### Requirement: Typed-field validation returns canonical values

Typed-field validation MUST allow consumers to validate a candidate JSON value for a declared scalar or compound field and obtain the canonical typed value or canonical compound JSON value, or a validation failure that preserves field identity, processing path, nested path when applicable, value kind, and constraint or shape reason. Source-specific coercion and source priority MUST remain consumer-owned before typed-field validation is called.

#### Scenario: Candidate value is canonicalized

- **WHEN** a consumer validates the candidate JSON value `true` accepted by a declared boolean field
- **THEN** typed-field validation returns the canonical boolean typed value
- **THEN** the consumer can serialize the canonical value into its own inspection, config-source, or request representation

#### Scenario: Candidate value fails constraints

- **WHEN** a consumer validates a candidate JSON value that violates a declared numeric range
- **THEN** typed-field validation returns a failure with the field identity, processing path, received value kind, and range reason
- **THEN** the consumer maps the failure into its own diagnostic boundary

#### Scenario: Compound candidate fails nested shape

- **WHEN** a consumer validates a candidate JSON array whose item object is missing a required declared member
- **THEN** typed-field validation returns a failure with the field identity, processing path, nested item/member path, received value kind, and shape reason
- **THEN** the consumer maps the failure into its own config-source diagnostic boundary

### Requirement: Processing-path projections preserve consumer ownership

Processing-path metadata projections MUST NOT own source priority, CLI flags, config command shape, adapter selection, public diagnostic code identity, protocol envelope shape, output rendering, or adapter-id namespace policy. Those policies MUST remain with the consuming capability.

#### Scenario: Consumer maps validation result

- **WHEN** typed-fields returns a validation failure for a config processing path
- **THEN** typed-fields provides field-level failure facts
- **THEN** the consuming config or navigation owner chooses the public diagnostic code, source priority behavior, source-specific coercion, and output projection
