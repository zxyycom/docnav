# Verification Record

## Status

- Change: `enforce-native-test-evidence-coverage`
- Audit gate: `Proceed`
- Implementation and single-track cutover: complete
- Current strict closure: passed, 536 NativeTestEntry machine cases and 27 Evidence Claims
- Full workspace verification: no failed checks; one non-blocking quality check warning

This record distinguishes deterministic acceptance from non-blocking quality observation. The
quality warning is not reported as a pass: it contains 19 warnings without accepted reasons,
including 17 changed/regression warnings in the new test-evidence implementation. The complete
records remain in the generated `artifacts/docnav-quality/` report; no warning was hidden by adding
an acceptance reason.

## Fixed skill distributions

### ast-grep v1

- Source: exact upstream release fixed in `audit.md`
- Files: 6
- Directory fingerprint:
  `sha256:8957af003ca667e987db9e42e7f76e8f6813a0fe9f7e87a09ce4454424de0d44`
- Verification: `scripts/test-evidence/toolchain.test.ts` checks the complete relative file list
  and fingerprint.

### test-evidence-review v8

- Owner: Docnav project
- Baseline: upstream v7 distribution fixed in `audit.md`
- Files: 10
- Directory fingerprint:
  `sha256:0c9c19156d2605694b0177897590e3e8cc8b444893d91a921535eaaf3c11fa94`
- Self-updater: none; future upstream sync requires explicit three-way review.

| Relative path | SHA-256 |
| --- | --- |
| `SKILL.md` | `ba27d776038e469a9020974c79b927331b81ece27bb4496790838156fe6249a2` |
| `agents/openai.yaml` | `12818dbb151940816929733731418f5a9caf7a97b02c610ae6ea9570b9873f0e` |
| `references/evidence-contract.md` | `0dc15bfdee345e113380c4f58c79efa233ffd3b8bfca994bf43bd00b35062643` |
| `schemas/claim-topic-catalog.schema.json` | `77182d8ab45d3f46a5ca3013ead606955102d293e489cdc7974ff2cfd411b9d0` |
| `schemas/evidence-claim.schema.json` | `20f77cb66659cbc2c53e9a24ddd78f08be995d77fb1e7e939a40227c4a768ede` |
| `schemas/native-test-entry.schema.json` | `c8dae5e83a82d1b351baddf896468ac4c58df7be2eb487e2d08bf5629deb1883` |
| `schemas/native-test-inventory.schema.json` | `2fccdad7a89ea4a64e783853ba1bc44e5d4b9ce0913403f304d8a0fbee70175d` |
| `schemas/test-evidence-index.schema.json` | `cbbb107e8d9a2c630c79af519996d474129741f6ea6441cd9f49fa00bfe56c88` |
| `scripts/test-evidence-catalog.d.mts` | `2443d5710e4c6d3bddeda4373a7b8d24b2d2038afd094aa52bd9976e56bcc96d` |
| `scripts/test-evidence-catalog.mjs` | `0733898a8779c69a9726b5499d7e8fef3ad6589aff8cac04477a5fe8b95edc1e` |

## Migration and closure evidence

The migration comparison materialized Git commit `a645ba1` in a repository-external temporary
directory and compared that tree with `migration-map.json` and the current inventory:

| Evidence | Result |
| --- | --- |
| Old source set | 431 unique paths; exact equality with the restored v7 Markdown set |
| Old source fingerprint | `sha256:16323dac67692c02d180b427d64f60842dd727e86281abdf05642c1d78fdcea8` |
| Entry outcomes | 426 unique current `entryKey` targets; 5 obsolete v7 catalog self-tests terminated |
| Claim outcomes | 27 migrated; 396 templates terminated for no information gain; 3 candidates terminated because the owner requirement was not explicit |
| Audit omissions | All 81 Rust and all 7 full-audit Bun omissions occur exactly once in the current inventory; this includes the original 3 Bun baseline omissions |
| Current universe | 536 entries = 426 mapped old entries + 88 audited omissions + 22 new/replacement entries |
| Net change from audit runtime | 536 = 519 audit-time entries - 5 replaced v7 self-tests + 22 current v8/toolchain/selector tests |

The same temporary tree exercised the complete rollback unit rather than only restoring data:

1. restored the v7 skill, 431 case Markdown files, topic table, index, validator, stable docs and
   AGENTS from `a645ba1`;
2. verified every restored case path against `migration-map.json`;
3. verified the v8 inventory, Claim topic, project wrapper and ast-grep skill were absent; and
4. ran the restored v7 strict validator successfully:
   `Test evidence check passed: 11 topic(s), 431 test case(s).`

The temporary tree was removed after the drill. The baseline commit and migration map remain the
recovery inputs.

## Focused verification

| Command | Result |
| --- | --- |
| `bun run test:test-evidence-rules` | Passed: 9 rule tests covering supported positive, nearest negative and unsupported shapes. |
| `bun run test:test-evidence` | Passed: 16 catalog, schema, query, change-report and toolchain boundary tests. |
| `bun run verify:test-evidence-toolchain` | Passed: minimal install and frozen reinstall used `--offline`; installed CLI reported `ast-grep 0.45.0`. |
| `uv run --with pyyaml python /home/dev/.codex/skills/.system/skill-creator/scripts/quick_validate.py .codex/skills/test-evidence-review` | Passed. `uv` supplied PyYAML outside the repository because the host Python did not provide it. |
| `bun test scripts/docs/test-evidence-validation.test.ts` | Passed: 5 closure/inventory tests covering one-to-one closure, static-only, runtime-only, duplicate, missing, orphan and stale states. |
| `bun test test/tools/smoke-harness.test.ts` | Passed: 9 smoke harness tests, including exact stable-ID leaf selection. |
| `bun run test-evidence -- sync --write --root .` | Passed: rebuilt 536 entries (391 Cargo, 128 Bun, 17 smoke), 11 topics, 27 Claims. |
| `bun run test-evidence -- check --root .` | Passed with the same counts and no diagnostics. |
| `bun run validate:docs` | Passed: test evidence, decisions, JSON/schema/examples and 49 Markdown link files. |
| `bun run typecheck:scripts` | Passed. |
| `bun run lint:scripts` | Passed. |
| `openspec validate enforce-native-test-evidence-coverage --type change --json --strict --no-interactive` | Passed with no issues. |

## Release boundary

The host canonical package path was built and verified:

| Command | Result |
| --- | --- |
| `bun run package:docnav` | Passed for `0.1.0-beta.1`, `x86_64-unknown-linux-gnu`; canonical package contains exactly 3 files. |
| `bun run verify:docnav-package` | Passed exact file-set, manifest, size and checksum validation. |
| `bun run smoke:docnav-package` | Passed 50 real CLI commands from the packaged `docnav`. |

The package contains only `docnav`, `manifest.json` and `SHA256SUMS.txt`; it contains no external
ast-grep executable or project rule. The toolchain boundary tests also prove only
`scripts/test-evidence/ast-grep.ts` invokes that developer executable. This local change validation
did not build the Windows release target and is not a public release baseline.

## Full workspace verification

`bun run verify:docnav-workspace` completed all 15 checks:

- 14 passed, including Cargo format, test, clippy, docs validators, OpenSpec, TypeScript
  typecheck/lint, release script tests, quality internal tests and development CLI smoke.
- 1 reported `warning`: the full quality observation had 19 warnings without accepted reasons,
  17 of them changed/regressions in the new implementation.
- 0 failed.

The largest new observations are cohesive but long discovery/inventory/CLI functions. They do not
change the strict acceptance result, and this first implementation does not suppress them with
quality configuration. A later code-simplification change can split those internal functions
without changing the v8 contract.

## Final single-track search

The final scoped search found no live use of `test-evidence-topics.json`, v7 independent-case
instructions, source markers, `Entry` / `Contract` / `Proves` templates, or v7 paths in stable
docs, AGENTS, code, the v8 skill, or other active changes. Archived changes and this migration
record retain historical wording intentionally.

The live evidence layout contains exactly 27 Claim Markdown files and three JSON files:
`claim-topics.json`, `native-test-inventory.json` and `test-evidence-index.json`. External
ast-grep references are limited to the pinned dependency declaration, the sole developer wrapper,
its callers, and boundary tests.
