# parallel-task-runner

Parallel task scheduling for TypeScript script tooling.

## Use

Import from `src/index.ts`.

This internal module provides task normalization, dependency graph validation, concurrency, mutex scheduling, and lifecycle hooks. It imports the adjacent `../foundation` internal module.

## Dependency completion

`dependsOn` waits for completion, not a consumer-defined success value. A dependency completes after its `execute` and `onComplete` handlers resolve. Resolved result values are opaque to the scheduler, so a result such as `{ ok: false }` still unblocks dependents. If `onStart`, `execute`, or `onComplete` rejects, `runParallelTasks` rejects and no additional tasks are scheduled.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
