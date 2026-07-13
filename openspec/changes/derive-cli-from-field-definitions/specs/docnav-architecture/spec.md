## ADDED Requirements

### Requirement: Project field authoring stays above generic typed fields

Docnav MUST place project-specific field authoring in a shared layer above `docnav-typed-fields`. The layer MUST own Docnav extension payloads、field builder extensions and framework-neutral projection views while leaving canonical field validation and opaque extension storage in typed-fields.

Adapter contracts and navigation MAY depend on the project authoring layer to declare fields. The layer MUST NOT depend on Clap、core command construction or adapter implementations; core/framework companions consume its derived views through higher-level integration so dependency direction remains acyclic.

#### Scenario: Common and adapter fields share one project builder

- **WHEN** navigation declares a common field and an adapter definition declares a native field
- **THEN** both can use the `docnav-field-authoring` builder extension
- **THEN** both produce the same project projection shape for downstream consumers

#### Scenario: Framework dependency stays above declarations

- **WHEN** core converts a project field projection into Clap arguments
- **THEN** Clap-specific code remains in core or the Clap companion
- **THEN** `docnav-field-authoring` and adapter contracts remain framework-neutral
