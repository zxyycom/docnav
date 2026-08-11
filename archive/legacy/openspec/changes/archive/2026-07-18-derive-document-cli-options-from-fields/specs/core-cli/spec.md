## ADDED Requirements

### Requirement: Core composes static commands with field-derived document options

Core CLI MUST continue to own root/subcommand topology、fixed positionals、management commands、core-owned static arguments、help/version side-effect boundaries and output/process mapping. For each document operation, core MUST augment that static shape with the operation-scoped canonical CLI field set supplied by navigation and MUST obtain normalized typed/invalid CLI candidates through the Clap companion.

Core MUST derive generated argument flag、identity、value capture、help/value name and canonical accepted/default display from the field projection. Every projected flag MUST map to exactly one canonical field identity before parsing. Generated-to-static conflicts、ambiguous generated locators and incompatible projections MUST fail deterministically before document dispatch.

#### Scenario: Generate a common document option

- **WHEN** the current operation field set contains `adapter`、`page`、`limit`、`pagination` or `output` with public CLI processing metadata
- **THEN** core registers and captures that option through the field projection
- **THEN** accepted values、default、value kind and canonical validation facts remain owned by the field declaration

#### Scenario: Scope adapter option help by operation

- **WHEN** a registry adapter native option is declared for outline but not read
- **THEN** generated outline help includes the option and generated read help omits it
- **THEN** both outcomes derive from declaration applicability

#### Scenario: Reject an ambiguous generated flag

- **WHEN** a projected document option conflicts with a fixed positional、core-owned flag、help/version flag or a common/native declaration from any registry adapter in the same operation
- **THEN** command construction returns a deterministic declaration/internal failure before parsing or dispatch
- **THEN** no ambiguous command shape is exposed

#### Scenario: Preserve non-document command behavior

- **WHEN** caller invokes help、version、config、init、doctor or adapter inspection
- **THEN** the existing static command owner handles that surface
- **THEN** document field projection is not evaluated for that command

#### Scenario: Lexical preflight consumes derived facts only

- **WHEN** core retains a lexical argv preflight to preserve strict token boundaries、repair context or existing spelling
- **THEN** its document option locators and cardinality come mechanically from the static command shape and canonical CLI field projection
- **THEN** field semantics remain in the canonical declaration

#### Scenario: Preserve output and process behavior

- **WHEN** document option parsing or selected resolution succeeds or fails
- **THEN** the existing output-context、diagnostic projection、stdout/stderr and exit mapping owners receive equivalent outcomes
- **THEN** this field derivation requirement does not redefine output modes or failure framing
