# parallel-task-runner

Docnav-neutral task definition, normalization, dependency graph validation, concurrency, mutex scheduling, and lifecycle hook runner.

## Public Source Entrypoint

- `src/index.ts`

The runner imports foundation helpers by source path from `../foundation/src/index.ts`. This private manifest is for local tooling and is not an npm publish contract.

## Runtime Prerequisites

- Bun for tests.
- `tsgo` and ESLint from the parent workspace dependencies.
- A pinned `scripts/tools/foundation/` checkout adjacent to this toolkit.

## Verification

- `bun run --cwd scripts/tools/parallel-task-runner typecheck`
- `bun run --cwd scripts/tools/parallel-task-runner lint`
- `bun run --cwd scripts/tools/parallel-task-runner test`

## Integration

Docnav imports `scripts/tools/parallel-task-runner/src/index.ts` directly. Docnav workspace profiles and check definitions remain in `scripts/docnav-workspace/**`.
