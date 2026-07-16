# foundation

TypeScript foundation helpers for script tooling.

## Use

Import from `src/index.ts`.

This internal module provides process, Git, path, fs, JSON, CSV, NDJSON, argument, error, and type-guard helpers. Consumers import its source directly inside the Docnav repository; this is not an npm package contract.

## Focused checks

Run these commands from this directory:

- `bun run typecheck`
- `bun run lint`
- `bun run test`
