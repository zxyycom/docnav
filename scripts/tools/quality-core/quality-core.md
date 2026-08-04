# Quality core module

Quality scanning core for TypeScript script tooling.

## Use

Import from `src/index.ts`.

This internal module provides quality schema/types, code-area classification, scanner adapters, metrics aggregation, warnings, reports, baseline/cache primitives, and `runQualityScan`. Callers provide repository-specific paths, globs, thresholds, tools, and scan options through typed config.

`metrics.json` schema `0.5.0` keeps `duplicateCode` as an array and requires
`duplicateCodeMeasurement.status` to qualify that value. The status is `measured`,
`skipped-by-profile`, `unavailable`, or `error`; only `measured` makes an empty array
mean that the configured scan found no duplicate fragments. Validation rejects a
missing measurement object, a non-object value, or an unknown status.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
