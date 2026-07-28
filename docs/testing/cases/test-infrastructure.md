# test-infrastructure

## Case AUX-PARALLEL-RUNNER-001: Parallel task runner 保持调度契约

Owner: `scripts/tools/parallel-task-runner/parallel-task-runner.md#use`

Entities:
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > does not limit concurrency when no explicit concurrency is provided`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > expands nested task groups with inherited metadata and group dependencies`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > normalizes task metadata and supports task.run as the execution body`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > rejects duplicate ids and unknown dependencies`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > rejects invalid task list metadata at the normalization boundary`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > respects an explicit concurrency limit`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > runs independent tasks concurrently but serializes matching mutexes`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > waits for onComplete while treating resolved result values as opaque`
- `bun|scripts/tools/parallel-task-runner/test/index.test.ts|parallel task runner > waits for topological dependencies before starting dependent tasks`

Proves:
- task normalization、concurrency、mutex serialization、dependency completion ordering 和 nested task expansion 保持稳定。
- resolved result 对 scheduler 保持 opaque；consumer status field 不阻塞 dependent，且 dependent 在 `onComplete` 完成后启动。
- 非法 list metadata、重复 task id 和未知 dependency failure 保持可诊断。

## Case AUX-SCRIPT-VALUE-PARSING-001: Shared script path normalization 与正整数解析稳定

Owner: `scripts/tools/foundation/foundation.md#use`

Entities:
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > normalizes backslashes in paths`
- `bun|scripts/tools/foundation/test/foundation.test.ts|script foundation > parses strict positive integers`

Proves:
- Shared script helpers 把反斜杠规范化为 slash path，并在 public boundary 只接受严格正整数。

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

Proves:
- independent smoke tasks 可以并发运行，同时 command count 按 report 隔离。
- 失败 task、nested task group、默认 runner 的 stdout/stderr command record 和 plain-text child environment 保持预期 audit result shape。
- core smoke 在 caller-owned base 下创建唯一 run child；失败 task 执行时 project cwd 已存在，结束后只删除 owned child 并保留 caller-owned base。

## Case AUX-SMOKE-HARNESS-002: Core smoke config fixture helper 不修改 checked-in fixture

Owner: `docs/testing.md#cli-smoke`

Entities:
- `bun|test/smoke/core/fixtures/project.test.ts|core smoke fixture projects > copies config fixtures before mutable config cases write`

Proves:
- mutable config cases 把 config fixture 安装到 `.tmp/docnav/smoke/` 的 CLI project wrapper，写入副本不改变 checked-in fixture。

## Case AUX-TEST-EVIDENCE-CATALOG-001: Semantic Case catalog parses and queries bounded evidence

Owner: `docs/testing/case-maintenance.md#查询与验证`

Entities:
- `bun|scripts/test-evidence/catalog.test.ts|diagnoses malformed Case structure and stable identity conflicts`
- `bun|scripts/test-evidence/catalog.test.ts|parses and queries topic-grouped semantic Cases`
- `bun|scripts/test-evidence/catalog-cli.test.ts|returns a query failure status at the CLI boundary`

Proves:
- Topic-grouped Markdown is parsed into semantic Cases with stable IDs, exact owner/entity mappings, and bounded topic/query/show projections.
- Malformed Case structure 与重复 stable identity 保持可区分；black-box CLI 在文档约定的 exit boundary 返回 query failure。

## Case AUX-TEST-EVIDENCE-CLOSURE-001: Current test entities close against semantic Case mappings

Owner: `docs/testing/case-maintenance.md#全树闭合`

Entities:
- `bun|scripts/test-evidence/closure.test.ts|closes current test entities against the union of Case mappings`

Proves:
- Static discovery and runner reports must describe the same complete current entity set.
- The union of explicit Case mappings covers every current entity and rejects references to entities absent from that closed set.

## Case AUX-TEST-EVIDENCE-DISCOVERY-001: Runner profile discovery is explicit and reproducible

Owner: `docs/testing/case-maintenance.md#全树闭合`

Entities:
- `bun|scripts/test-evidence/discovery/profile.test.ts|loads one versioned and sorted supported runner profile`
- `bun|scripts/test-evidence/discovery/profile.test.ts|parses stable Cargo and Bun runner reports without inferring missing fields`
- `bun|scripts/test-evidence/discovery/bun-files.test.ts|expands Bun test roots with include, ignore and supplemental files`
- `bun|scripts/test-evidence/discovery/bun-files.test.ts|rejects invalid, empty and redundant Bun test surfaces`

Proves:
- One versioned runner profile expands Cargo and Bun surfaces deterministically and parses runner reports without inventing missing identity fields.
- Bun include, ignore, and supplemental paths reject invalid, empty, redundant, or escaping discovery surfaces.

## Case AUX-TEST-EVIDENCE-TOOLCHAIN-001: Test discovery uses the repository-locked AST toolchain

Owner: `docs/testing.md#脚本与工具依赖`

Entities:
- `bun|scripts/test-evidence/toolchain.test.ts|does not invoke the external ast-grep executable outside the developer wrapper`
- `bun|scripts/test-evidence/toolchain.test.ts|uses the repository-locked ast-grep CLI through the project wrapper`

Proves:
- Test discovery 只通过 project wrapper 调用 repository-locked ast-grep CLI。

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
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters known success noise from terminal-visible output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > filters quality timing details from terminal-visible output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > formats completion lines and durations for streaming output`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > keeps actionable failure output after filtering known success noise`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > maps quality warning markers to warning check status`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > parses verification profile arguments`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > prepares development binary env with isolated copied executables`
- `bun|scripts/docnav-workspace/verify.test.ts|workspace verifier configuration > prints visible warning output immediately after completion lines`
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
- Verifier 隔离 development binaries，并且只清理自己复制的 artifacts。
