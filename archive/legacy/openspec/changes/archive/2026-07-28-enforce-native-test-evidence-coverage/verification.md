# Verification Record

## Status

- Change: `enforce-native-test-evidence-coverage`
- Current-system audit gate: `Proceed`
- Current strict closure: passed
- Native entries: 536
  - Cargo: 391
  - Bun: 128
  - smoke: 17
- Evidence Claims: 23
- Used Claim topics: 4
- Full workspace verification: 14 passed, 1 warning-status check, 0 failed
- Quality warnings without accepted reasons: 17

## Current-only evidence model

The final current evidence set has no compatibility reader or second authority:

- source and runner reports own entry existence;
- every closed Entry has exactly one generated machine case;
- inventory and query index are rebuildable current projections;
- 23 Claims use semantic IDs and precise current owner references;
- 4 Claims that only repeated their test names were removed;
- the topic catalog contains only the 4 categories used by current Claims;
- the change no longer stores a per-case historical mapping or recovery protocol.

Scoped search found no version-tagged contract wording, opaque historical Claim
prefixes, per-case migration dependency or compatibility reader in the stable
test-evidence skill, project wrapper, evidence directory, stable maintenance
documents, or related active changes.

## Skill and toolchain integrity

| Component | Current result |
| --- | --- |
| project `ast-grep` skill | 6 tracked files; fingerprint `sha256:8957af003ca667e987db9e42e7f76e8f6813a0fe9f7e87a09ce4454424de0d44` |
| project `test-evidence-review` skill | 10 tracked files; fingerprint `sha256:38fa7fe98879b5f1bae042734fc4c92817228ba7c0479d22e3a12ab2846dc7f8`; no updater |
| external developer CLI | `@ast-grep/cli` exactly `0.45.0`; offline frozen install passed |
| invocation boundary | only `scripts/test-evidence/ast-grep.ts` invokes the external executable |

## Focused verification

| Command | Result |
| --- | --- |
| `bun run test:test-evidence-rules` | Passed: 9 positive, nearest-negative and unsupported-shape rule tests. |
| `bun run test:test-evidence` | Passed: 16 catalog, schema, query, change-report, profile/root boundary and toolchain tests. |
| `bun run verify:test-evidence-toolchain` | Passed: minimal and frozen reinstalls were offline; CLI reported `ast-grep 0.45.0`. |
| `uv run --with pyyaml python /home/dev/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/test-evidence-review` | Passed. |
| `bun run test-evidence -- sync --write --root .` | Passed: rebuilt 536 Entries, 23 Claims and the current index. |
| `bun run test-evidence -- check --root .` | Passed with the same counts and no diagnostics. |
| `bun run validate:docs` | Passed: evidence, decisions, JSON/schema/examples and 45 Markdown link files. |
| `bun run typecheck:scripts` | Passed. |
| `bun run lint:scripts` | Passed. |
| `openspec validate enforce-native-test-evidence-coverage --type change --json --strict --no-interactive` | Passed with no issues. |

## Release boundary

| Command | Result |
| --- | --- |
| `bun run package:docnav` | Passed for `0.1.0-beta.1`, `x86_64-unknown-linux-gnu`; package contains exactly 3 files. |
| `bun run verify:docnav-package` | Passed exact file-set, manifest, size and checksum validation. |
| `bun run smoke:docnav-package` | Passed 50 real CLI commands. |

The canonical package contains only `docnav`, `manifest.json` and
`SHA256SUMS.txt`; it contains no external ast-grep executable, project rule or
test-evidence skill.

## Full workspace verification

`bun run verify:docnav-workspace` completed all 15 checks:

- 14 passed, including whitespace, Cargo fmt/test/clippy, TypeScript
  typecheck/lint, docs validators, OpenSpec, release tests, quality internal
  tests and development CLI smoke;
- 1 reported `warning`: quality full check;
- 0 failed.

The quality report keeps 17 warnings without accepted reasons visible, including
7 changed/regression warnings. This correction only separates newly added
inventory boundary validation from inventory generation, comparison and I/O;
remaining discovery/inventory/CLI observations stay visible for separate review.
