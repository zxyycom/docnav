# test-infrastructure

## Case AUX-PARALLEL-RUNNER-001: Parallel task runner 保持调度契约

Owner: `scripts/tools/parallel-task-runner/README.md#use`

Entities:
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > does not limit concurrency when no explicit concurrency is provided`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > expands nested task groups with inherited metadata and group dependencies`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > normalizes task metadata and supports task.run as the execution body`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > rejects duplicate ids and unknown dependencies`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > rejects invalid task list metadata at the normalization boundary`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > respects an explicit concurrency limit`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > runs independent tasks concurrently but serializes matching mutexes`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > schedules an explicitly prepared task list`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > waits for onComplete while treating resolved result values as opaque`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > waits for topological dependencies before starting dependent tasks`

Proves:
- task normalization、concurrency、mutex serialization、dependency completion ordering 和 nested task expansion 保持稳定。
- resolved result 对 scheduler 保持 opaque；consumer status field 不阻塞 dependent，且 dependent 在 `onComplete` 完成后启动。
- prepare strategy、invalid list metadata、duplicate id 和 unknown dependency failure 保持可诊断。

## Case AUX-SCRIPT-VALUE-PARSING-001: Shared script value parsing rejects ambiguous inputs

Owner: `scripts/tools/foundation/README.md#use`

Entities:
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > parses JSON values and normalizes slash paths`
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > parses strict positive integers`

Proves:
- Shared script helpers parse JSON values, normalize slash paths, and accept only strict positive integers at their public boundaries.

## Case AUX-SMOKE-HARNESS-001: Smoke harness 正确记录 task 和 command 输出语义

Owner: `docs/testing.md#cli-smoke`

Entities:
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > creates and cleans only the owned core smoke run directory after task failure`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > records default runner stdout and stderr on command records`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > records failed task results without stopping other independent tasks`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > runs default runner commands with plain text output environment`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > runs independent smoke tasks concurrently and keeps per-task command counts isolated`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > runs nested case tasks but records only the parent smoke report`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > selects one smoke leaf by its stable id and preserves the parent report`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > uses DOCNAV_SMOKE_CONCURRENCY at the smoke scheduling boundary`
- `bun|test/tools/smoke-harness.test.ts|smoke harness task scheduling > validates smoke concurrency values`

Proves:
- independent smoke tasks 可以并发运行，同时 command count 按 report 隔离。
- failed task、nested task group、默认 runner 的 stdout/stderr command record、plain-text child environment 和 concurrency validation 保持预期 audit result shape。
- `DOCNAV_SMOKE_CONCURRENCY` 只在 smoke scheduling boundary 作为默认并发输入生效；直接解析 `undefined` 不得隐式读取全局环境变量。
- core smoke 在 caller-owned base 下创建唯一 run child；失败 task 执行时 project cwd 已存在，结束后只删除 owned child 并保留 caller-owned base。

## Case AUX-SMOKE-HARNESS-002: Core smoke config fixture helper 保持配置/文档分层

Owner: `docs/testing.md#cli-smoke`

Entities:
- `bun|test/smoke/core/fixtures/project.test.ts|core smoke fixture projects > copies config fixtures before mutable config cases write`
- `bun|test/smoke/core/fixtures/project.test.ts|core smoke fixture projects > uses semantic config fixtures with the shared Markdown document`

Proves:
- config cases 使用按语义命名的 checked-in JSON fixture，并复用共享 Markdown document path。
- mutable config cases 把 config fixture 安装到 `.tmp/docnav/smoke/` 的 CLI project wrapper，写入副本不改变 checked-in fixture。

## Case AUX-TEST-EVIDENCE-CATALOG-001: Semantic Case catalog parses and queries bounded evidence

Owner: `docs/testing/case-maintenance.md#查询与验证`

Entities:
- `bun|scripts/test-evidence/catalog.test.ts|diagnoses malformed Case structure and stable identity conflicts`
- `bun|scripts/test-evidence/catalog.test.ts|parses and queries topic-grouped semantic Cases`
- `bun|scripts/test-evidence/catalog.test.ts|uses distinct exit statuses for discovery, runner, Case, and query failures`

Proves:
- Topic-grouped Markdown is parsed into semantic Cases with stable IDs, exact owner/entity mappings, and bounded topic/query/show projections.
- Malformed Case structure, duplicate stable identity, and command-layer discovery, runner, Case, or query failures remain distinguishable at their owning boundary.

## Case AUX-TEST-EVIDENCE-CLOSURE-001: Current test entities close against semantic Case mappings

Owner: `docs/testing/case-maintenance.md#全树闭合`

Entities:
- `bun|scripts/test-evidence/catalog.test.ts|closes current test entities against the union of Case mappings`

Proves:
- Static discovery and runner reports must describe the same complete current entity set.
- The union of explicit Case mappings covers every current entity and rejects references to entities absent from that closed set.

## Case AUX-TEST-EVIDENCE-DISCOVERY-001: Runner profile discovery is explicit and reproducible

Owner: `docs/testing/case-maintenance.md#全树闭合`

Entities:
- `bun|scripts/test-evidence/catalog.test.ts|loads one versioned and sorted supported runner profile`
- `bun|scripts/test-evidence/catalog.test.ts|parses stable Cargo and Bun runner reports without inferring missing fields`
- `bun|scripts/test-evidence/discovery/bun-files.test.ts|expands Bun test roots with include, ignore and supplemental files`
- `bun|scripts/test-evidence/discovery/bun-files.test.ts|rejects invalid, empty and redundant Bun test surfaces`

Proves:
- One versioned runner profile expands Cargo and Bun surfaces deterministically and parses runner reports without inventing missing identity fields.
- Bun include, ignore, and supplemental paths reject invalid, empty, redundant, or escaping discovery surfaces.

## Case AUX-TEST-EVIDENCE-TOOLCHAIN-001: Test discovery uses the repository-locked AST toolchain

Owner: `docs/testing.md#脚本与工具依赖`

Entities:
- `bun|scripts/test-evidence/toolchain.test.ts|does not invoke the external ast-grep executable outside the developer wrapper`
- `bun|scripts/test-evidence/toolchain.test.ts|keeps the development ast-grep executable outside canonical release components`
- `bun|scripts/test-evidence/toolchain.test.ts|uses the repository-locked ast-grep CLI through the project wrapper`

Proves:
- Test discovery invokes the repository-locked ast-grep CLI only through the project wrapper and keeps that developer dependency outside release components.

## Case AUX-WORKSPACE-PROCESS-001: Shared process wrapper plain-text environment 稳定

Owner: `docs/tooling.md#子进程输出环境`

Entities:
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > detects failed process results`
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > runs child processes with plain text output environment`

Proves:
- shared process wrapper 在 sync 和 async child process 入口覆盖 caller-provided color env，统一注入 plain-text output environment。
- Failed child-process results remain distinguishable from successful results without throwing away their observable status.

## Case AUX-WORKSPACE-VERIFY-001: Workspace verifier 保持 required/full profile 语义

Owner: `docs/testing.md#统一验证入口`

Entities:
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters cargo trybuild success noise from successful cargo test output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters decision success output from docs validator failures`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters package manager echoes from successful script checks`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters quality timing details from terminal-visible output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters successful semantic Case ledger output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > formats completion lines and durations for streaming output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > keeps actionable failure output after filtering known success noise`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > maps quality warning markers to warning check status`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > parses verification profile arguments`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > prepares development binary env with isolated copied executables`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > prints visible warning output immediately after completion lines`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > rejects invalid leaf and group check definitions`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > removes copied development binary artifacts`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > reports environment setup errors as failed check results`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > resolves verifier concurrency only when a limit is configured`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > schedules docs validation through one executable check`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > separates required and full verification profiles`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > suppresses all passed output even when a success line is not configured`

Proves:
- Required and full verifier profiles keep distinct membership while sharing one normalized check-report pipeline.
- Successful subprocess noise is filtered, but actionable warning and failure diagnostics remain visible with stable completion and duration summaries.
- The required profile includes semantic test-ledger validation and quick quality checks; the full profile replaces quick quality with its broader quality gate.
- Verifier configuration rejects invalid leaf/group definitions, isolates development binaries, and cleans only its copied artifacts.
