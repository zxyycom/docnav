本 spec delta 定义共享脚本工具能力的新增要求：Docnav 脚本中可复用的工具内核必须能被提取为子仓库或可发布包，同时 Docnav 专属策略必须留在 Docnav 集成层。

## ADDED Requirements

### Requirement: 共享脚本工具必须分离通用内核和 Docnav 策略

Shared script tooling MUST expose reusable execution, task, quality, verifier, and packaging primitives separately from Docnav-specific policy. The reusable primitives MAY be used by Docnav and other projects, but Docnav-owned paths, package names, artifact layouts, validators, quality policies, verification profiles, release components, CLI package scripts, and owner documentation MUST remain outside shared defaults.

#### Scenario: Docnav 策略不进入共享默认值

- **WHEN** a shared script toolkit is extracted from Docnav scripts
- **THEN** the toolkit exposes Docnav-neutral functions, types, task models, scanner adapters, or packaging helpers
- **THEN** Docnav-specific values such as `artifacts/docnav-quality`, `.cache/docnav`, `product: "docnav"`, `binName: "docnav"`, Docnav OpenSpec validators, and Docnav schema/example paths remain outside shared default behavior

#### Scenario: Docnav 集成层拥有配置

- **WHEN** Docnav integrates an extracted toolkit
- **THEN** Docnav passes repository root, artifact paths, task definitions, quality config, release components, output filters, and validation rules through a documented API
- **THEN** the shared toolkit does not infer those values by inspecting the Docnav repository layout

### Requirement: 共享脚本工具必须支持多包和多子仓库拆分

Shared script tooling MUST allow independent package or repository boundaries by domain, maturity, dependency set, and release cadence. A single all-encompassing scripts repository MUST NOT be required for extraction.

#### Scenario: 能力按生命周期分开提取

- **WHEN** foundation helpers, parallel task scheduling, quality observability, workspace verification, and release packaging helpers have different maturity or dependency needs
- **THEN** the extraction plan can place them in separate packages or repositories
- **THEN** each package or repository records its exports, dependencies, tests, version policy, and changelog owner

#### Scenario: 首批不覆盖所有脚本

- **WHEN** only a subset of Docnav script tooling has a clear reusable boundary
- **THEN** Docnav extracts only that subset
- **THEN** unrelated Docnav quality, verifier, release, or validation behavior remains in Docnav scripts

### Requirement: 共享脚本工具必须通过 typed config 接收可变策略

Shared script tooling MUST represent configurable behavior through typed configuration, explicit task definitions, adapter objects, or function parameters. Shared code MUST NOT implicitly depend on Docnav repository root discovery, fixed glob sets, fixed artifact directories, fixed Cargo workspace shape, fixed package manager scripts, fixed OpenSpec layout, or fixed protocol/schema/example files.

#### Scenario: Quality core 接收调用方配置

- **WHEN** quality observability code is extracted
- **THEN** scanner execution, metrics aggregation, warning generation, baseline/cache handling, and report generation use a caller-provided quality config
- **THEN** code areas, include/exclude globs, thresholds, accepted warnings, artifact directories, tool command paths, and repository root are supplied by Docnav wrapper or configuration

#### Scenario: Verifier core 接收任务定义

- **WHEN** workspace verifier behavior is extracted
- **THEN** task graph validation, profile/check execution, output filtering, completion reporting, and concurrency handling operate on caller-provided task definitions and profile definitions
- **THEN** Docnav required/full check lists, Cargo/OpenSpec/docs commands, and ignore/allow regex remain in Docnav-owned definitions

### Requirement: Docnav 集成层必须保持既有可观察行为

Docnav wrappers MUST preserve existing command names, script entry points, artifact locations, warning status semantics, verification profile composition, release package outputs, and validation ownership unless a separate change explicitly modifies those contracts. Extraction MUST NOT change Docnav CLI, protocol, adapter, schema, examples, or document output behavior.

#### Scenario: Existing package scripts remain meaningful

- **WHEN** Docnav migrates a local script implementation to an extracted toolkit
- **THEN** existing entries such as `typecheck:scripts`, `lint:scripts`, `quality:check`, `verify:docnav-workspace`, release package scripts, and validator scripts keep their documented role
- **THEN** any changed command output, artifact path, status mapping, or validation scope is treated as a separate Docnav-owned behavior change

#### Scenario: Product contracts are not affected

- **WHEN** shared script tooling is extracted
- **THEN** Docnav `docnav` CLI operations, adapter routing, protocol envelopes, readable output, schemas, and examples remain unchanged
- **THEN** script extraction does not add fields, flags, adapter payloads, or output modes to Docnav product contracts

### Requirement: 提取必须具备发布准备和验证证据

Extracted script tooling MUST provide package-level verification, publishing readiness, version or pin strategy, changelog entries for breaking changes, and Docnav migration evidence before a toolkit boundary is accepted.

#### Scenario: Package and publishing readiness exist

- **WHEN** a toolkit is extracted into a package or repository
- **THEN** it declares package metadata, public exports, README, runtime prerequisites, typecheck/lint/test commands, changelog or version policy, and release notes
- **THEN** those checks run without requiring Docnav-specific docs, schemas, examples, OpenSpec changes, or release artifacts

#### Scenario: Docnav migration evidence exists

- **WHEN** Docnav adopts an extracted toolkit
- **THEN** Docnav runs the relevant script typecheck, lint, focused script tests, workspace verifier, quality scan, release package test, or validator commands for the touched surface
- **THEN** the recorded evidence compares migration-relevant command output, artifact paths, warning statuses, reports, release metadata, and exit behavior with the pre-extraction behavior
