本 delta 将三个共享脚本 Git 子仓库收敛为 Docnav 主仓库内模块，并把独立仓库交付要求替换为根验证要求。

## RENAMED Requirements

- FROM: `共享脚本工具必须使用 Git 子仓库边界`
- TO: `共享脚本工具必须使用单仓库内部边界`
- FROM: `提取必须具备交付准备和验证证据`
- TO: `内部工具必须纳入根验证链路`

## MODIFIED Requirements

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
