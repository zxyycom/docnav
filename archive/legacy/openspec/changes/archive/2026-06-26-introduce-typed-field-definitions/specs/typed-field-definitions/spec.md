## ADDED Requirements

### Requirement: Typed field definitions define reusable field identity and path metadata
Typed field definitions MUST define stable field identity, structured JSON path metadata, field validation metadata, required/default metadata, and field-level constraint metadata without depending on a consumer-specific transport, CLI surface, or schema document layout.

#### Scenario: Field metadata is registered independently of consumer semantics
- **WHEN** a consumer registers a typed field definition
- **THEN** the definition exposes a canonical field identity
- **AND** the definition exposes structured JSON path segments that can derive display or schema paths
- **AND** the definition exposes field validation metadata, required/default metadata, and field-level constraints
- **AND** consumer-owned layers remain responsible for CLI argv parsing, configuration source precedence, operation binding, manifest/probe policy, protocol envelope shape, and readable output framing

### Requirement: Field validation groups type and field constraints
Typed field definitions MUST use a structured field validation object to group value kind, Rust value type, and field-level constraints, and MUST NOT require arbitrary fallback validation functions in the initial core.

#### Scenario: Field validation is structured metadata
- **WHEN** a consumer defines validation for a typed field
- **THEN** value kind, Rust value type, and constraints such as enum, numeric range, string regex, and string/array length are registered through one field validation object
- **AND** value kind constructors such as int, string, or num declare the value kind and Rust value type without requiring a separate value kind declaration
- **AND** string enum validation can be declared from a real Rust enum metadata source while exposing JSON enum values in field metadata
- **AND** declared enum constraints fail during definition build when the effective allowed value set is empty
- **AND** string enum validation can only be declared from a real Rust enum metadata source
- **AND** numeric range constraints use explicit open/closed bounds
- **AND** integer range constraints use integer bounds without converting through floating point
- **AND** number range constraints use floating number bounds
- **AND** numeric `min` and `max` constraints can be declared independently for single-sided ranges
- **AND** floating number range bounds that are present are finite
- **AND** string validation can include regex and character length constraints
- **AND** array validation can include element count constraints
- **AND** array item schema and object property schema validation remain outside the initial typed field definition core
- **AND** the validation metadata can be used for decode, validation, and schema metadata view
- **AND** arbitrary function validators are not part of the initial typed field definition core

### Requirement: Typed field validation reports attributed field failures
Typed field validation MUST decode and validate only the target field value, and every validation failure MUST carry enough attribution for the consumer owner to map the failure to its own stable diagnostics.

#### Scenario: Invalid field values preserve attribution
- **WHEN** a typed field decoder receives a missing required value, wrong JSON value kind, disallowed enum value, range violation, regex mismatch, length violation, or invalid default metadata
- **THEN** validation fails with the field identity
- **AND** validation fails with the structured field path
- **AND** validation fails with a machine-readable reason for the field-level violation
- **AND** the consumer owner maps that failure to its user-facing error category, diagnostic text, stdout/stderr placement, or protocol error envelope

### Requirement: Schema metadata remains a field-level view
Typed field definitions MUST expose schema metadata facts for downstream tooling while leaving complete JSON Schema document ownership to the existing schema and surface owners.

#### Scenario: Schema tooling consumes field facts without taking ownership
- **WHEN** schema, docs, or fixture tooling reads typed field metadata
- **THEN** it can obtain field-level facts such as type, requiredness, null-as-absent policy, default metadata, enum values, numeric range constraints, regex constraints, length constraints, and path metadata
- **AND** static default metadata is typed consistently with field validation
- **AND** non-finite floating number static defaults fail during definition set build with a reason that explains Rust `f64` can represent non-finite values but JSON numbers cannot
- **AND** runtime default sources are not part of the typed field definition API
- **AND** existing schema and surface owners remain responsible for complete public schema files, `$ref` layout, schema `$id` values, protocol envelopes, readable output schemas, and example validation policy

### Requirement: Definition sets reject duplicate field identities
Typed field registration MUST reject duplicate canonical field identities in a definition set.

#### Scenario: Duplicate identities fail set build
- **WHEN** two registrations use the same canonical field identity
- **THEN** definition set build fails even when the duplicate declarations have equivalent field semantics
- **AND** the duplicate identity failure includes field identity attribution
- **AND** duplicate enum string aliases are deduplicated in effective field metadata

### Requirement: Consumers keep their owner semantics
Consumers of typed field definitions MUST reuse field metadata without inheriting semantics owned by other consumers or public contract surfaces.

#### Scenario: Standard parameter resolution consumes field metadata
- **WHEN** standard parameter resolution consumes typed field metadata
- **THEN** source mapping, merge order, source info, passthrough, warning policy, and operation argument binding remain standard-parameter responsibilities

#### Scenario: JSON contract validation consumes field metadata
- **WHEN** manifest, probe, or protocol JSON validation consumes typed field metadata
- **THEN** JSON contract ownership, schema parity, semantic validation, stable error mapping, and envelope behavior remain with the relevant adapter/protocol owner

### Requirement: Definition sets provide typed-field projections
Typed field definitions MUST support building a set of field definitions that validates field consistency and exposes typed-field projections without performing consumer-owned resolution.

#### Scenario: Definition set builds field projections
- **WHEN** a consumer builds a typed field definition set
- **THEN** the set validates every field definition and rejects duplicate field identities
- **AND** the set can expose an `extract_without_default` function, an `extract_with_static_defaults` function, a `validate_without_default` function, a `validate_with_static_defaults` function, a value kind view, a typed default values object, and a schema metadata view
- **AND** a `FieldDefs` derive struct can produce a same-shaped typed values object after extracting an input JSON value
- **AND** nested typed values object shape is expressed through Rust struct fields marked with `#[field(group)]`
- **AND** each leaf Rust field explicitly declares presence policy and Rust value type through `T` or `Option<T>`
- **AND** generated code checks each leaf `#[field(...)]` expression against the matching `FieldDefBuilder<T>` type
- **AND** definition set build projects the declared `T`/`Option<T>` presence policy into leaf field metadata
- **AND** typed values object field access is expressed through generated Rust fields rather than runtime identity-string lookup
- **AND** dynamic identity-string field lookup is not part of the definition set API
- **AND** extract and validate failures use the same field validation error type
- **AND** the definition object is not itself the business parameter object
- **AND** typed values objects expose the Rust value type carried by the leaf field validation, such as `Option<i64>` for an optional integer field or `String` for a required string field
- **AND** a required `T` leaf fails when the input path is missing or the input value is JSON null
- **AND** an optional `Option<T>` leaf extracts missing input paths and JSON null values as `None`
- **AND** `extract_without_default` does not apply static defaults
- **AND** `extract_with_static_defaults` applies static defaults only when the input omits a field
- **AND** `extract_with_static_defaults` returns the same typed values object shape as `extract_without_default`
- **AND** the typed default values object follows the derive struct shape rather than the input JSON path
- **AND** default leaves without a static default are represented as typed `None`
- **AND** each leaf `FieldDef` builder supplies canonical field identity, structured field path, validation, and default metadata
- **AND** leaf field builders are not built during declaration; set build performs field build validation, declaration presence metadata projection, and default metadata validation
- **AND** field build failures from derive declarations preserve the struct field declaration path for attribution
- **AND** missing leaf field validation fails through the generated `FieldDefBuilder<T>` type check
- **AND** missing leaf field paths fail during set build
- **AND** non-finite floating number range bounds and empty open/closed ranges fail during set build
- **AND** non-finite floating number static defaults fail during set build with a reason that explains Rust `f64` can represent non-finite values but JSON numbers cannot
- **AND** built definition sets expose a typed `to_builder()` copy path that can statically override leaf field builders and rebuild a new read-only definition set
- **AND** the set does not perform standard parameter source merge, CLI argv parsing, operation binding, manifest/probe policy, or complete schema document generation
