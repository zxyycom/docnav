本 delta spec 定义 `replace-probe-traversal-with-inferred-routing` 对 `adapter-contract` 尚未应用的 Target：manifest 同时拥有 format、完整-basename suffix 与 exact-filename routing facts，固定 adapter strategy 完整删除 probe；它不表示 Current 主规范或实现已经迁移。

## MODIFIED Requirements

### Requirement: Adapter definition owns registry-facing adapter facts

Adapter definition, manifest, and descriptor metadata MUST describe adapter identity, supported format facts, pathname routing hints, capability declarations, and the linked strategy implementation. The adapter definition MUST be the registry-facing aggregation point for those adapter behavior facts. Each format descriptor MUST expose a project-owned normalized format identity, one or more dotted `extensions[]`, and a `filenames[]` array that MAY be empty. Each extension value MUST be a basename suffix with a leading dot, MAY contain additional dots, and MUST NOT contain a path separator. Filename values MUST contain one exact basename without a path separator and MUST NOT equal `.` or `..`. Automatic routing MUST compare exact filenames case-sensitively first. Otherwise it MUST compare the complete basename and extension suffixes after ASCII case normalization and MUST choose the longest matching suffix. Across the core static registry, one normalized format identity MUST resolve to at most one adapter definition; one ASCII-normalized suffix or one exact filename MUST resolve to at most one format identity within its hint kind. Registry construction, doctor, and release validation MUST reject exact duplicates before document routing. Different-length suffix overlap MUST remain valid and deterministic rather than be classified as a conflict. Exact filename and suffix are distinct hint kinds, so one basename MAY have an exact-filename mapping that overrides its generic suffix mapping.

The fixed adapter strategy interface MUST provide outline, read, find, and info functions and MUST NOT define a probe function, probe result, probe reason, probe version, or selection-detection hook. Adapter-private helpers MAY construct manifest or capability values, but shared layers MUST consume adapter behavior facts through the exported definition/factory. Adapter implementation source MUST remain a core static-registry fact. Caller-configurable document-operation parameter facts MUST come from the separate core catalog. Pathname matching MUST remain navigation-private routing mechanics and MUST NOT transfer document decode, parse, ref, or operation semantics away from the selected adapter.

#### Scenario: Core lists built-in adapters

- **WHEN** `docnav adapter list` inspects adapters
- **THEN** implementation source comes from the core static registry
- **THEN** manifest metadata describes adapter capability, normalized format support, complete-basename suffixes, and exact filenames
- **THEN** listing does not execute a selection probe
- **THEN** document-operation parameter facts remain in the separate core catalog

#### Scenario: Registry consumes one adapter definition

- **WHEN** a built-in adapter is registered with core
- **THEN** the registry receives one adapter definition containing identity, format descriptors, a linked strategy implementation, and optional capabilities
- **THEN** the fixed strategy interface provides the required operations and exposes no probe surface
- **THEN** caller-configurable parameter facts come from the core catalog

#### Scenario: Automatic routing uses manifest pathname hints

- **WHEN** navigation receives a document pathname without explicit adapter intent
- **THEN** registry-derived exact-filename and normalized-suffix lookup views map the complete basename to a manifest format identity
- **THEN** registry matching compares that identity exactly with definition format descriptors
- **THEN** an adapter definition is not executed merely to decide whether it is a candidate
- **THEN** registry order does not choose between equal format identities

#### Scenario: Exact filename overrides a generic suffix

- **WHEN** one complete basename matches a declared exact filename and also ends with another declared suffix
- **THEN** the exact filename mapping supplies the routing identity
- **THEN** the cross-kind overlap is not treated as a duplicate declaration

#### Scenario: Registry declares duplicate format identity

- **WHEN** two built-in adapter definitions declare the same normalized format identity
- **THEN** core registry construction, doctor, and release validation reject the registry
- **THEN** no document invocation uses registry order to choose one definition

#### Scenario: Registry declares duplicate pathname hint

- **WHEN** two format descriptors declare the same ASCII-normalized suffix or the same exact filename
- **THEN** core registry construction, doctor, and release validation reject the registry
- **THEN** runtime does not reinterpret the conflict as document ambiguity

#### Scenario: Compound suffix is more specific

- **WHEN** no exact filename matches and basename `model.schema.JSON` is checked against `.json` and `.schema.json`
- **THEN** ASCII normalization makes both suffixes eligible
- **THEN** `.schema.json` supplies the routing identity because it is the longest match
- **THEN** the overlap is not rejected as a duplicate pathname hint

#### Scenario: Adapter implementation uses private helpers

- **WHEN** an adapter implementation splits definition construction across private helper functions or modules
- **THEN** it exports one registry-facing definition or definition factory
- **THEN** registry, navigation, and dispatch consume adapter-owned behavior facts through that definition
- **THEN** core catalog remains the only parameter-definition input
