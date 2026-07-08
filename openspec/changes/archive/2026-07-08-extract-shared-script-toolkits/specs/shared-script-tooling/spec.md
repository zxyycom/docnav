本 spec delta 定义共享脚本工具能力的新增要求：Docnav 脚本中可复用的工具内核必须能被提取为 Git 子仓库，同时 Docnav 专属策略必须留在 Docnav 集成层。

## ADDED Requirements

### Requirement: 共享脚本工具必须分离通用内核和 Docnav 策略

Shared script tooling MUST expose reusable execution, task, and quality primitives separately from Docnav-specific policy. The reusable primitives MAY be used by Docnav and other projects, but Docnav-owned paths, artifact layouts, validators, quality policies, verification profiles, release components, CLI package scripts, and owner documentation MUST remain outside shared defaults.

#### Scenario: Docnav 策略不进入共享默认值

- **WHEN** a shared script toolkit is extracted from Docnav scripts
- **THEN** the toolkit exposes Docnav-neutral functions, types, task models, scanner adapters, metrics aggregation, warning/report generation, or cache helpers
- **THEN** Docnav-specific values such as `artifacts/docnav-quality`, `.cache/docnav`, `product: "docnav"`, `binName: "docnav"`, Docnav OpenSpec validators, and Docnav schema/example paths remain outside shared default behavior

#### Scenario: Docnav 集成层拥有配置

- **WHEN** Docnav integrates an extracted toolkit
- **THEN** Docnav passes repository root, artifact paths, task definitions, quality config, release components, output filters, and validation rules through a documented API
- **THEN** the shared toolkit does not infer those values by inspecting the Docnav repository layout

### Requirement: 共享脚本工具必须使用 Git 子仓库边界

Shared script tooling MUST use multiple Git toolkit repository boundaries for the first extraction. The first extraction MUST separate foundation helpers, parallel task scheduling, and quality core into three distinct toolkit repositories by domain, maturity, dependency set, and release cadence, and npm package publication MUST NOT be required for Docnav integration.

#### Scenario: 首批能力拆成多个子仓库

- **WHEN** foundation helpers, parallel task scheduling, and quality observability core are extracted
- **THEN** foundation helpers are placed under `scripts/tools/foundation/`
- **THEN** parallel task scheduling is placed under `scripts/tools/parallel-task-runner/`
- **THEN** quality observability core is placed under `scripts/tools/quality-core/`
- **THEN** each toolkit repository exposes its own public source entrypoint and records its dependencies, tests, Git revision policy, and changelog owner

#### Scenario: 首批不覆盖所有脚本

- **WHEN** only a subset of Docnav script tooling has a clear reusable boundary
- **THEN** Docnav extracts only that subset
- **THEN** workspace verifier profiles, release product config, validators, Docnav quality defaults, and CLI command entrypoints remain in Docnav scripts

### Requirement: 共享脚本工具必须通过 typed config 接收可变策略

Shared script tooling MUST represent configurable behavior through typed configuration, explicit task definitions, adapter objects, or function parameters. Shared code MUST NOT implicitly depend on Docnav repository root discovery, fixed glob sets, fixed artifact directories, fixed Cargo workspace shape, fixed package manager scripts, fixed OpenSpec layout, or fixed protocol/schema/example files.

#### Scenario: Quality core 接收调用方配置

- **WHEN** quality observability code is extracted
- **THEN** scanner execution, metrics aggregation, warning generation, baseline/cache handling, report generation, and quality scan orchestration use a caller-provided quality config and runtime options
- **THEN** code areas, include/exclude globs, thresholds, accepted warnings, artifact directories, cache directories, tool command paths, repository root, changed-file input, baseline policy, report options, and output adapter are supplied by Docnav caller, command entrypoint, or configuration
- **THEN** Docnav defaults such as `DOCNAV_*` environment variables, `artifacts/docnav-quality`, `.cache/docnav/quality`, `crates/**/*.rs`, `docs/examples/**`, and `docs/schemas/**` remain outside shared defaults

#### Scenario: Verifier core 接收任务定义

- **WHEN** workspace verifier behavior is extracted
- **THEN** task graph validation, profile/check execution, output filtering, completion reporting, and concurrency handling operate on caller-provided task definitions and profile definitions
- **THEN** Docnav required/full check lists, Cargo/OpenSpec/docs commands, and ignore/allow regex remain in Docnav-owned definitions

### Requirement: Docnav 集成层必须保持既有可观察行为

Docnav callers and command entrypoints MUST preserve existing command names, script entry points, artifact locations, warning status semantics, verification profile composition, release package outputs, and validation ownership unless a separate change explicitly modifies those contracts. Extraction MUST NOT change Docnav CLI, protocol, adapter, schema, examples, or document output behavior.

#### Scenario: Existing package scripts remain meaningful

- **WHEN** Docnav migrates a local script implementation to an extracted toolkit
- **THEN** existing entries such as `typecheck:scripts`, `lint:scripts`, `quality:check`, `verify:docnav-workspace`, release package scripts, and validator scripts keep their documented role
- **THEN** any changed command output, artifact path, status mapping, or validation scope is treated as a separate Docnav-owned behavior change

#### Scenario: Product contracts are not affected

- **WHEN** shared script tooling is extracted
- **THEN** Docnav `docnav` CLI operations, adapter routing, protocol envelopes, readable output, schemas, and examples remain unchanged
- **THEN** script extraction does not add fields, flags, adapter payloads, or output modes to Docnav product contracts

### Requirement: 提取必须具备交付准备和验证证据

Extracted script tooling MUST provide toolkit-repository-level verification, local tooling readiness, Git revision or pin strategy, changelog entries for breaking changes, and Docnav migration evidence before a toolkit boundary is accepted.

#### Scenario: Subrepository readiness exists

- **WHEN** a toolkit is extracted into one of the first-batch toolkit repositories
- **THEN** it declares a private tooling manifest if needed, public source entrypoint, README, runtime prerequisites, typecheck/lint/test commands, changelog or Git revision policy, and delivery notes
- **THEN** those checks run without requiring Docnav-specific docs, schemas, examples, OpenSpec changes, or release artifacts

#### Scenario: Docnav migration evidence exists

- **WHEN** Docnav adopts an extracted toolkit
- **THEN** Docnav runs the relevant script typecheck, lint, focused script tests, workspace verifier, quality scan, release package test, or validator commands for the touched surface
- **THEN** the recorded evidence compares migration-relevant command output, artifact paths, warning statuses, reports, quality artifacts, and exit behavior with the pre-extraction behavior
