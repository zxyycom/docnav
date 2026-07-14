## ADDED Requirements

### Requirement: Project field authoring and framework projection have explicit owners

Docnav MUST place project-specific field authoring in a shared layer above `docnav-typed-fields`. The layer MUST own `DocnavCliPresentation`、field builder extensions and framework-neutral `DocnavFieldProjection` views while leaving canonical field validation and immutable type-indexed extension storage in typed-fields.

Adapter contracts and navigation MAY depend on the project authoring layer to declare fields. The layer MUST NOT depend on Clap、core command construction、the Clap companion or adapter implementations.

`cli-config-resolution-clap` MUST own its framework-facing `ClapFieldSpec` input without depending on Docnav crates. Docnav core MAY depend on both sides and MUST own the mechanical `DocnavFieldProjection -> ClapFieldSpec` bridge. The bridge MUST copy already-derived facts without re-authoring flag、constraint、default、accepted-value、owner or operation semantics, so dependency direction remains acyclic.

#### Scenario: Common and adapter fields share one project builder

- **WHEN** navigation declares a common field and an adapter definition declares a native field
- **THEN** both can use the `docnav-field-authoring` builder extension
- **THEN** both produce the same project projection shape for downstream consumers

#### Scenario: Framework dependency stays above declarations

- **WHEN** core converts a project field projection into Clap arguments
- **THEN** core maps the Docnav view to the companion-owned input and the companion performs Clap-specific work
- **THEN** `docnav-field-authoring` and adapter contracts remain framework-neutral

#### Scenario: Independent companion does not depend on Docnav

- **WHEN** the `cli-config-resolution` workspace builds and tests independently
- **THEN** its Clap companion accepts its own consumer-neutral projection input
- **THEN** no package in that workspace depends on `docnav-field-authoring` or other main-workspace crates
