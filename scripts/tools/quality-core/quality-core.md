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

Runtime metrics validation accepts only non-array records for object fields and
non-empty strings for required metadata strings; truthy values of another runtime
type are invalid. Revision-input discovery reports an explicit `unavailable` scope with its
Git failure reason when diff or working-tree state cannot be obtained. It never uses
a revision snapshot to claim that an unknown diff is unchanged. The separate
one-commit changed-file collector may conservatively use the current revision file
set when `HEAD~1` does not exist, without changing the detection result. Automatic
comparison and changed-file channels remain unavailable unless the caller supplies
an explicit changed-files list; an explicit list becomes the effective available
scope rather than retaining an unrelated Git-discovery failure.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
