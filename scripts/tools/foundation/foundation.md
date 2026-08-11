# Foundation script module

TypeScript foundation helpers for script tooling.

## Use

Import from `src/index.ts`.

This internal module provides process, Git, path, fs, JSON, CSV, NDJSON, argument, error, and type-guard helpers. Consumers import its source directly inside the Docnav repository; this is not an npm package contract.

Recursive file discovery is fail-closed: if any requested directory cannot be read, `walkFiles` reports that directory as an error instead of returning a partial file set.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
