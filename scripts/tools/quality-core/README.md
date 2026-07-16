# quality-core

Quality scanning core for TypeScript script tooling.

## Use

Import from `src/index.ts`.

This internal module provides quality schema/types, code-area classification, scanner adapters, metrics aggregation, warnings, reports, baseline/cache primitives, and `runQualityScan`. Callers provide repository-specific paths, globs, thresholds, tools, and scan options through typed config.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
