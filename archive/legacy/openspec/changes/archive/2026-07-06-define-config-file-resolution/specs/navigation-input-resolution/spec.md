## 一句话核心

`docnav-navigation` SHALL load raw config sources only from core-supplied user/project config source descriptors; 当前 change 只在 `openspec/changes/define-config-file-resolution/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## 文档状态

本 spec delta 只描述 `navigation-input-resolution` capability 的新增要求。它不让 navigation 层重新发现 CLI flags、environment variables 或 project roots，也不改变 adapter-owned option validation。

## ADDED Requirements

### Requirement: Navigation loads config sources from descriptor paths with origin-aware absence semantics

`docnav-navigation` MUST treat project and user config files as raw source inputs loaded from core-supplied descriptors. Each descriptor MUST preserve the source level, resolved path and whether the path came from explicit CLI input or a default path. Missing default-path config sources MUST be absent without diagnostics. Missing, unreadable, invalid JSON, or non-object config sources selected through explicit CLI config path flags MUST produce a blocking config source diagnostic. Config path flag selection MUST NOT become a navigation parameter source value and MUST NOT alter the parameter merge priority `explicit > project > user > built_in`.

#### Scenario: Explicit config path failure is blocking

- **WHEN** core supplies a project or user config source descriptor selected by `--project-config` or `--user-config`
- **AND** the descriptor path is missing, unreadable, invalid JSON, or a top-level non-object JSON value
- **THEN** `docnav-navigation` returns a blocking config source diagnostic for that source level and path
- **THEN** lower-priority config sources or built-in defaults do not mask that source failure

#### Scenario: Default config path absence is non-blocking

- **WHEN** core supplies a project or user config source descriptor selected by default path resolution
- **AND** the descriptor path does not exist
- **THEN** `docnav-navigation` treats that config source as absent
- **THEN** it continues resolving declared parameters from remaining sources and built-in defaults

#### Scenario: Config path selection is separate from parameter priority

- **WHEN** core supplies explicit config file paths and the selected project config, selected user config and direct argv all provide candidates for the same declared parameter
- **THEN** `docnav-navigation` resolves the parameter value from direct argv first
- **THEN** project config still overrides user config
- **THEN** user config still overrides built-in defaults
- **THEN** the fact that a config file path was selected by CLI flag does not turn values inside that file into direct argv values

#### Scenario: Diagnostics preserve selected config path

- **WHEN** a present project or user config source contains an unknown field, unsupported selected-adapter option, type mismatch or range-invalid value
- **THEN** `docnav-navigation` reports the source level and selected config file path in the diagnostic details
- **THEN** diagnostics distinguish project config from user config even when both paths were supplied through CLI flags
