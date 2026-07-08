# foundation

Docnav-neutral TypeScript script helpers for process execution, process result modeling, Git commands, path/fs/json helpers, argument parsing, error messages, and type guards.

## Public Source Entrypoint

- `src/index.ts`

Consumers import source directly from a pinned Git checkout or subrepo path. This manifest is private tooling metadata and is not an npm publish contract.

## Runtime Prerequisites

- Bun for script and test execution.
- Node.js-compatible APIs provided by the repository toolchain.
- `tsgo` and ESLint from the parent workspace dependencies.
- `git` only for helpers that explicitly execute Git commands.

## Verification

- `bun run --cwd scripts/tools/foundation typecheck`
- `bun run --cwd scripts/tools/foundation lint`
- `bun run --cwd scripts/tools/foundation test`

## Integration

Docnav callers import foundation modules from `scripts/tools/foundation/src/**` directly. Callers pass cwd, paths, commands, and environment explicitly.
