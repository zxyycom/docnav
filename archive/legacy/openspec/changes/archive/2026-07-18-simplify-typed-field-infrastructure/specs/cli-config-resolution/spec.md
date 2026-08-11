## MODIFIED Requirements

### Requirement: Framework Adapter Boundary

Framework-specific extraction MUST remain outside the framework-independent resolution core. The workspace MUST retain the structured-config companion; environment extraction MAY remain in the core because it requires no external framework. Docnav core CLI MUST own a private, bounded mapping for the flags and value kinds used by its closed catalog. Core CLI, direct input, structured-config, env, and defaults MUST all emit candidates through the same resolution source model.

#### Scenario: Core CLI maps its closed catalog

- **WHEN** core CLI registers or reads document arguments
- **THEN** it maps only fields from the core-owned document-operation parameter catalog
- **THEN** it emits canonical candidates for those fields
- **THEN** it implements only projections required by catalog entries in the release

#### Scenario: Use structured config companion with canonical parameters

- **WHEN** navigation passes a structured config document and the core-owned `FieldDefSet` containing config-path strategies to the structured-config companion
- **THEN** the companion returns candidates for declared canonical field identities
- **THEN** the resolution core receives them through the same candidate/source model used by direct input and env extraction

#### Scenario: Extract environment variables without ambient global state

- **WHEN** a consumer passes an iterable collection of environment key/value pairs and a canonical parameter set to the env extractor
- **THEN** the extractor reads only declared environment locators
- **THEN** tests and core-owned consumers can supply input without requiring the extractor to own process-global environment access

#### Scenario: Environment locator activates extraction

- **WHEN** a catalog field has no environment locator
- **THEN** the env extractor emits no candidate for that field even if a similarly named environment key exists

#### Scenario: Enabled environment locator emits a candidate

- **WHEN** an owner change adds an environment locator to a catalog field
- **THEN** the extractor may emit the declared env candidate through the canonical source model

### Requirement: Docnav Hard Cutover Boundary

Docnav MUST consume exactly one core-owned closed parameter catalog through canonical typed-field extraction and resolution APIs while preserving existing Docnav CLI, config, merge, accepted-value, adapter behavior, protocol, diagnostic, and output behavior. Resolution MUST produce the standard typed values and provenance needed to construct operation input. Core MAY execute catalog-selected static validation before dispatch; adapter strategies MAY execute or repeat semantic validation after dispatch. The catalog MUST be the only caller-configurable document-operation parameter definition source.

#### Scenario: Resolve common and adapter-scoped core fields

- **WHEN** Docnav resolves navigation fields and a selected adapter's applicable parameters
- **THEN** it keeps untagged entries plus entries whose exact adapter-id marker matches the selected adapter, then applies the operation binding
- **THEN** it passes those canonical field definitions directly to extraction and resolution
- **THEN** it materializes the values needed for core-defined standard operation input

#### Scenario: Preserve Docnav source priority and merge behavior

- **WHEN** Docnav maps explicit input, project config, user config, built-in defaults, and any currently enabled sources into ordered sources
- **THEN** resolution matches Docnav's documented priority behavior and each canonical field's merge strategy
- **THEN** a field with an enabled env locator uses `explicit > env > project > user > built_in`
- **THEN** a field without an env locator emits no env candidate and retains `explicit > project > user > built_in`
- **THEN** Docnav remains the owner of request construction, diagnostic-code mapping, and output projection

#### Scenario: Map pre-parsed input through a core-owned source adapter

- **WHEN** Docnav already holds parsed direct CLI input before resolution
- **THEN** a Docnav-private source adapter may traverse core catalog processing metadata and emit public `Source` / `SourceCandidate` values
- **THEN** it uses a bounded direct-input source representation derived from catalog metadata
- **THEN** field type, core constraint, default, validation, and merge metadata continue to come from the catalog

#### Scenario: Resolution leaves adapter semantics to the strategy

- **WHEN** a core-owned adapter-scoped field has been resolved and materialized but its remaining validity depends on adapter or document context
- **THEN** resolution preserves the typed value and provenance needed by standard operation input
- **THEN** it does not require the adapter semantic rule to become a foreign field declaration

#### Scenario: Runtime uses one catalog path

- **WHEN** the core catalog path passes Docnav equivalence tests
- **THEN** every document operation resolves caller-configurable parameters through that catalog
- **THEN** no alternative parameter-definition source participates in runtime resolution

### Requirement: Root Cargo Workspace Membership

Typed-fields, resolution core, and the structured-config companion MUST remain ordinary members of the Docnav root Cargo workspace under `crates/shared/`, using the root dependency metadata, lockfile, and validation surfaces. `cli-config-resolution` MUST remain the primary resolution entry for canonical parameter types. Shared packages and facade exports MUST correspond to retained production capabilities and consumers.

#### Scenario: Build retained packages from the root workspace

- **WHEN** a maintainer checks out Docnav and runs the root Cargo workspace checks
- **THEN** typed-fields, resolution core, and the structured-config companion resolve from `crates/shared/`
- **THEN** they use the root lockfile without a nested workspace checkout or dependency-prefetch path

#### Scenario: Workspace contains the retained package set

- **WHEN** workspace metadata and package dependencies are inspected
- **THEN** each retained shared package has a production capability or consumer identified by this change
- **THEN** facade exports match that retained package set

#### Scenario: External publication still requires separate approval

- **WHEN** the retained packages build successfully inside the Docnav root workspace
- **THEN** workspace membership creates no external publication or compatibility contract
- **THEN** any future external consumer or publication requires a separate change
