# Drift Recovery and Rules File Review

Use this reference when context appears stale, overloaded, contradictory, or when updating Docnav agent rules.

## Drift Signals

Run a drift recovery pass when one of these signals appears:

- The task role is unclear or no `docs/navigation.md` role path has been selected.
- Loaded context includes broad docs, generated files, schemas, examples, or OpenSpec changes that do not serve the current task.
- A claim conflicts with file paths, refs, CodeGraph results, command output, or tests.
- The agent is about to edit outside the stated ownership scope.
- Markdown or large docs were read without the repository-declared bounded navigation path when one is available.

## Evidence Gate

Treat each important assumption as valid only when it has one of these sources:

- A relevant `AGENTS.md` or local rules file.
- A section read through `docs/navigation.md` role routing.
- A bounded doc read through the repository-declared Markdown/navigation command.
- CodeGraph search/node/callers/callees/impact output.
- A filtered `rg` / `rg --files` fallback with path constraints.
- A schema/example only when the task is validating fields, output shape, or tests.
- A command output with the precise command and focused result.

## Recovery Flow

1. Restate the conflict using paths, refs, symbols, commands, or failing lines.
3. Reload only the authoritative source for that layer.
4. Update the assumption ledger with `known`, `inferred`, and `open` entries.
5. Resume with the smallest context packet that explains the current decision.
6. Ask the user only when repo evidence cannot resolve the conflict or scope rules block the necessary edit.

## Rules File Checklist

When creating or reviewing `AGENTS.md`, project skills, or other rules files, check for:

- Repository-specific purpose instead of generic agent advice.
- Clear scope, ownership, and parallel worker safety.
- `docs/navigation.md` as the document entry point.
- CodeGraph first for code structure, with filtered search as fallback.
- Markdown read path through the repository-declared navigation command; avoid hardcoding build output paths.
- OpenSpec reads gated by OpenSpec work, audit, validation, or explicit user request.
- Schema/example reads gated by contract validation or tests.
- Positive checkpoints such as context packet, assumption ledger, evidence gate, and bounded verification.
- Direct links to references for long templates or checklists.

## Common Fixes

- Replace broad "read all docs" instructions with role-based routing through `docs/navigation.md`.
- Replace repeated "do not drift" reminders with a context packet plus assumption ledger.
- Move long examples, review prompts, and failure mode lists from `SKILL.md` into `references/*.md`.
