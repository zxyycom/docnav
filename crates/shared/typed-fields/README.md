# docnav-typed-fields

Canonical field mechanics shared by Docnav's Rust workspace. This package owns `FieldDef` /
`FieldDefSet`, processing locators, value kinds, constraints, static defaults, merge strategies,
layered validation, typed values, provenance-ready metadata, and all-or-nothing materialization.

## Document CLI metadata status

- **Current:** optional framework-neutral help, value-name, and Boolean input encoding attach to
  `ProcessStrategy::cli_flag` and remain part of the canonical field facts. Typed-fields rejects
  invalid or duplicate processing locators and incompatible Boolean encoding; config-only fields
  remain valid without CLI metadata.

Accepted values, defaults, constraints, and merge semantics remain canonical field facts.
Schema and processing views borrow those facts instead of copying them. Framework command
topology, operation applicability, diagnostics, source priority, and output policy remain
consumer-owned.

The main entry points are:

- `FieldDef::builder` and `FieldDefSet::builder` for direct, consumer-owned construction.
- `FieldValidation` for standard type materialization and reusable validation rules.
- `MergeStrategy::{Replace, Append, MapMerge, DenyConflict}` for canonical merge behavior.
- `FieldValueMap` and `FieldValue` for typed materialization.

Consumers can import the direct builders through this crate or the primary
[`cli-config-resolution`](../cli-config-resolution/README.md) facade. Framework-specific parsing
stays with the owning consumer.
