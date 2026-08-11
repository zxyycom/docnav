## ADDED Requirements

### Requirement: Markdown consumes core-defined adapter-scoped parameters

Markdown adapter MUST implement the fixed adapter strategy interface and consume the closed operation-specific input defined by the shared operation contract and populated from core catalog resolution. For `max_heading_level`, core MUST own its public flag/config path, standard integer type, public range `1..=6`, default `3`, source resolution, outline/find binding, exact `docnav-markdown` marker, pre-dispatch validation policy, and compile-time standard-input binding. Markdown MUST receive the resolved integer through the typed input field/accessor rather than generic parameter or protocol lookup. Markdown MUST own how that integer filters headings and MAY repeat the range check or perform additional algorithmic semantic validation before use. Markdown schema, examples, and strategy checks remain validation material rather than independent parameter declarations.

#### Scenario: Markdown parameter is configured

- **WHEN** project or user config provides valid `options.docnav-markdown.max_heading_level`
- **THEN** navigation resolves the source through core catalog
- **THEN** Markdown receives the standard integer accessor without parsing protocol `Options`
- **THEN** outline/find apply that value through the standard strategy input

#### Scenario: Core rejects the current public range

- **WHEN** caller input provides `max_heading_level` outside `1..=6`
- **THEN** core-owned input resolution reports the diagnostic before dispatch
- **THEN** the existing caller-visible behavior remains compatible

#### Scenario: Markdown defensively repeats a semantic check

- **WHEN** a well-typed standard input reaches Markdown through an internal or deliberately deferred validation path
- **THEN** Markdown may validate `max_heading_level` before applying the heading strategy
- **THEN** rejection maps to a compatible diagnostic
- **THEN** the check does not declare the parameter or participate in source resolution

## REMOVED Requirements

### Requirement: Markdown native options are declared adapter inputs

**Reason**: Markdown remains the format behavior owner, while the static core release defines every accepted caller-configurable document-operation parameter.

**Migration**: Move `max_heading_level` declaration, exact `docnav-markdown` marker, outline/find binding, source behavior, and standard-input binding to core catalog; preserve its CLI/config/protocol behavior; pass the resolved typed value to Markdown; retain any strategy-side check needed for defensive or algorithmic correctness.
