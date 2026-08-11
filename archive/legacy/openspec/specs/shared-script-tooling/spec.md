# shared-script-tooling Specification

## Purpose
TBD - created by archiving change extract-shared-script-toolkits. Update Purpose after archive.
## Requirements
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

### Requirement: 共享脚本工具必须使用单仓库内部边界

Foundation helpers, parallel task scheduling, and quality engine code MUST be stored as ordinary tracked source inside the Docnav repository under `scripts/tools/foundation/`, `scripts/tools/parallel-task-runner/`, and `scripts/tools/quality-core/`. These directories MUST be available after a normal clone and MUST NOT require Git submodule initialization, gitlink revision pins, or independently checkoutable toolkit repositories. The three domain directories, existing source APIs, and focused tests MUST remain available, and the focused tests MUST be covered by the root validation chain.

#### Scenario: 普通 clone 包含全部脚本源码

- **WHEN** a maintainer clones the Docnav repository without recursive submodule options
- **THEN** foundation, parallel task runner, and quality engine source and tests are present
- **THEN** root tooling can typecheck, lint, test, and execute them without fetching another repository

#### Scenario: 领域目录是内部边界

- **WHEN** root tooling imports one of the three script domains
- **THEN** the import uses a repository-owned internal path
- **THEN** the directory does not require an independent checkout or revision pin

### Requirement: 内部工具必须纳入根验证链路

Internal script tooling MUST be verified through the Docnav root package, TypeScript, ESLint, test, workspace verifier, quality, and release-package surfaces that cover its current responsibilities. A local manifest, configuration file, README, or focused command MAY remain when it serves a current in-repository caller or focused maintenance workflow. Root validation MUST cover the applicable source and tests but need not invoke every local command. Separate toolkit-repository readiness, Git revision strategy, isolated checkout verification, and standalone repository evidence MUST NOT be required. Observable behavior remains governed by the existing `Docnav 集成层必须保持既有可观察行为` requirement.

#### Scenario: 根配置证明内部模块

- **WHEN** a maintainer runs the documented root script checks
- **THEN** foundation, scheduler, and quality engine source participate in the applicable typecheck, lint, and focused tests
- **THEN** no nested package installation or submodule checkout is required

#### Scenario: 局部维护入口仍有仓库内用途

- **WHEN** a local manifest, configuration, README, or focused command serves a current in-repository caller or maintenance workflow
- **THEN** it may remain as a local entrypoint while root validation covers the applicable source and tests
- **THEN** retaining it does not create an independent checkout, release, or compatibility contract
