# Delivery Notes

## Extraction Inventory

Shared core:

- `scripts/tools/foundation/src/index.ts`: process result/runner, generic Git helpers, path/fs/json/csv/ndjson/args/error/type-guard helpers.
- `scripts/tools/parallel-task-runner/src/index.ts`: task definitions, normalization, graph validation, concurrency, mutex scheduler, lifecycle hooks.
- `scripts/tools/quality-core/src/index.ts`: quality schema/types, code-area classification, scanner adapters/parsers, aggregation, warning/report generation, baseline/cache primitives, and `runQualityScan`.

Docnav callers/config:

- `scripts/quality/scan.ts`
- `scripts/quality/config.ts`
- `scripts/quality/args.ts`
- `scripts/docnav-workspace/**`, `scripts/release-package/**`, and `scripts/tools/validators/**`

Not extracted in this change:

- Docnav validators for protocol/schema/examples/docs because they encode Docnav product contracts.
- Docnav quality defaults because they own `DOCNAV_*`, `artifacts/docnav-quality`, code areas, warning policy, and tool command defaults.
- Workspace verifier profiles because they own Cargo/OpenSpec/docs command composition and output filters.
- Release package config because it owns `product: "docnav"`, `binName: "docnav"`, target selection, manifest, and artifact layout.

## Subrepo Boundaries

| Subrepo | Public entrypoint | Manifest | Verification | Integration path |
| --- | --- | --- | --- | --- |
| `scripts/tools/foundation/` | `src/index.ts` | `package.json` | `bun run typecheck`, `bun run lint`, `bun run test` | `scripts/tools/foundation/src/**` direct imports |
| `scripts/tools/parallel-task-runner/` | `src/index.ts` | `package.json` | `bun run typecheck`, `bun run lint`, `bun run test` | `scripts/tools/parallel-task-runner/src/**` direct imports |
| `scripts/tools/quality-core/` | `src/index.ts` | `package.json` | `bun run typecheck`, `bun run lint`, `bun run test` | `scripts/tools/quality-core/src/**` direct imports |

Each subrepo has `README.md`, `CHANGELOG.md`, `DELIVERY.md`, private manifest, local tsconfig, runtime prerequisites, verification scripts, and initial revision policy.

## Baseline And Migration Evidence

Pre-migration baseline:

- `bun run typecheck:scripts`: passed.
- `bun run lint:scripts`: passed.
- Focused script tests: 31 passed.
- `bun run quality:check`: exited 0 with warning status, 435 files, 3340 functions, 6 warnings, artifacts under `artifacts/docnav-quality/quick/`.
- `bun run verify:docnav-workspace:required`: exited 0 with workspace status `warning`, 8 passed checks, 1 quality warning check, log `.log/verify/workspace/latest.log`.
- `bun run test:release-package-scripts`: 8 passed.
- `bun run test:validators`: 2 passed.

Post-migration evidence:

- Each subrepo `typecheck`, `lint`, and focused `test`: passed.
- `bun run typecheck:scripts`: passed.
- `bun run lint:scripts`: passed.
- `bun run quality:test`: 35 passed.
- `bun run test:workspace-verifier`: 35 passed.
- `bun run test:release-package-scripts`: 8 passed.
- `bun run test:validators`: 2 passed.
- `bun run quality:check`: exited 0 with warning status, 443 files, 3360 functions, 11 warnings, artifacts under `artifacts/docnav-quality/quick/`.
- `bun run verify:docnav-workspace`: exited 0 with workspace status `warning`, 13 passed checks, 1 quality warning check, 0 failed checks, log `.log/verify/workspace/latest.log`.
- `openspec validate "extract-shared-script-toolkits" --type change --json --strict --no-interactive`: passed.

Behavior comparison:

- Preserved command names: `typecheck:scripts`, `lint:scripts`, `quality:check`, `verify:docnav-workspace`, release package script tests, validator tests.
- Preserved quality artifact paths: `artifacts/docnav-quality/quick/metrics.json`, `report.md`, `warnings.ndjson`, `warnings-all.ndjson`.
- Preserved quality warning status: standalone `quality:check` exits 0 and reports `Quality check status: warning`; workspace full profile exits 0 and reports summary status `warning`.
- Expected metric inventory change: quality quick now scans shared toolkit source under `scripts/tools/**`, so file/function/warning counts changed after the implementation moved into toolkit repository boundaries. Exit behavior, status semantics, quick artifact paths, quick changed-warning count, and quick regression-warning count remained stable.

## Pin And Rollback

Current equivalent pin for all three first-batch shared script tools: parent repository revision `f95b84b89e4d75efca3cff42e9249c690b59aff9`.

Docnav does not use npm package versions for integration state. Rollback path is to reset the subrepo revision/pin and restore the affected Docnav caller imports or command entrypoint implementation to the last verified local implementation. The first-batch capabilities not moved remain candidates for later OpenSpec changes only when a reusable, Docnav-neutral boundary is explicit.

## Product Contract Audit

This change did not edit Rust CLI, adapter routing, protocol envelopes, readable output, schemas, examples, or product fixtures. The touched surface is limited to shared TypeScript tooling extraction, Docnav config integration, root TS/ESLint/package script coverage, tooling/testing docs, and OpenSpec delivery material.
