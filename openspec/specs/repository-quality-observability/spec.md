# repository-quality-observability Specification

## Purpose
Define how the Docnav repository collects, stores, and reports non-blocking code quality observation snapshots. This capability owns engineering observability artifacts and leaves Docnav CLI, adapter, protocol, output, schema, and example product contracts to their product owners.

## Requirements
### Requirement: Quality observation produces non-blocking snapshots
Repository quality observation MUST generate auditable snapshots without blocking normal product verification by default.

#### Scenario: Quality scan runs
- **WHEN** the quality scan command runs successfully
- **THEN** it writes a snapshot artifact
- **THEN** normal product contract validation remains separately owned

### Requirement: Quality tools have layered responsibilities
Quality observation MUST separate metric collection, normalization, report generation, baseline comparison, and CI presentation.

#### Scenario: Report is generated
- **WHEN** raw metric tools produce outputs
- **THEN** normalization creates stable intermediate data
- **THEN** reporting consumes normalized data rather than parsing tool-specific text ad hoc

### Requirement: Snapshots cover the initial core metrics
Quality snapshots MUST cover the repository's agreed initial metrics such as lint findings, complexity, duplication, size, and scan metadata where those tools are configured.

#### Scenario: Snapshot is inspected
- **WHEN** a maintainer opens the machine snapshot
- **THEN** it contains metric sections for the configured tool set
- **THEN** absent tools are represented explicitly rather than silently hidden

### Requirement: Reports support machine and human consumers
Quality reports MUST provide machine-readable data and human-readable summaries without requiring one consumer to parse the other's representation.

#### Scenario: CI publishes report
- **WHEN** CI runs quality observation
- **THEN** machine data is available for automation
- **THEN** a concise human summary is available for review

### Requirement: Baseline delta is explicit opt-in
Quality baseline comparison MUST be explicit opt-in and MUST distinguish observation from blocking quality gates.

#### Scenario: Baseline comparison requested
- **WHEN** a maintainer enables baseline delta reporting
- **THEN** the report compares against the selected baseline
- **THEN** blocking behavior remains absent until a separate gate explicitly owns enforcement

### Requirement: Observation is configuration-driven
Quality observation MUST use repository configuration to decide scan inputs, exclusions, output locations, and reporting behavior.

#### Scenario: Scan boundary is configured
- **WHEN** a path is excluded by quality configuration
- **THEN** quality observation keeps the scan inside the configured boundary
- **THEN** the report preserves enough metadata to audit the boundary

### Requirement: Observation reserves future gate boundaries
Quality observation MUST label reserved metadata for future gates, trends, or reports as observational until a separate gate capability defines enforcement.

#### Scenario: Future gate field exists
- **WHEN** a snapshot includes a field reserved for future gate usage
- **THEN** the report labels it as observational metadata
- **THEN** no blocking behavior is inferred from that field
