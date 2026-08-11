本 delta 将 CLI/config resolution packages 从独立子仓库迁入 Docnav 根 Cargo workspace，并保持 canonical field 与 resolution 语义不变。

## RENAMED Requirements

- FROM: `### Requirement: Independent Cargo Workspace Repository`
- TO: `### Requirement: Root Cargo Workspace Membership`

## MODIFIED Requirements

### Requirement: Root Cargo Workspace Membership

The typed-field and CLI/config resolution packages MUST be ordinary members of the Docnav root Cargo workspace under `crates/shared/` and MUST use the root workspace dependency metadata, lockfile, build, test, lint, and documentation surfaces. The root workspace MUST provide separate typed-fields, typed-fields macros, resolution core, clap companion, and structured-config companion packages with their existing package names, and `cli-config-resolution` MUST remain the primary resolution entry that re-exports canonical parameter types. An independently checkoutable workspace repository MUST NOT be required.

#### Scenario: 从根 workspace 构建全部 packages

- **WHEN** a maintainer checks out Docnav and runs the root Cargo workspace checks
- **THEN** typed-fields, its proc-macro, resolution core, clap companion, and structured-config companion resolve from `crates/shared/`
- **THEN** they use the root lockfile without a nested workspace checkout or dependency-prefetch path

#### Scenario: Docnav 使用内部 resolution packages

- **WHEN** Docnav protocol, adapter contracts, navigation, or core consumes canonical fields or resolution behavior
- **THEN** Cargo resolves the packages as root-workspace path dependencies
- **THEN** repository placement changes do not alter the runtime parameter semantics owned by the other requirements in this capability

#### Scenario: 外部发布仍需独立批准

- **WHEN** the packages build successfully inside the Docnav root workspace
- **THEN** workspace membership creates no external publication or compatibility contract
- **THEN** any future external consumer or publication requires a separate change
