# Context Packet and Assumption Ledger

Use this reference when a Docnav task needs handoff, compaction recovery, parallel worker coordination, or a precise context packet before a risky change.

## Context Packet

Keep the packet short enough for the next agent to act without rereading the whole repository.

```text
DOCNAV CONTEXT PACK
Task:
- One sentence describing the goal and ownership scope.

Allowed scope:
- Paths that may be edited.
- Paths that are read-only or owned by other workers.

Rules:
- Relevant AGENTS.md or local rules file constraints.
- Markdown read rule and tool constraints.

Role and spec:
- Role selected from docs/navigation.md.
- Main spec refs or bounded sections read through the repository-declared navigation command.

Relevant files:
- File path: why it matters.
- Related tests, fixtures, schemas, or examples when applicable.

Evidence:
- CodeGraph symbols or call paths.
- Filtered searches used as fallback.
- Command outputs or failing lines that define the current state.

Assumptions:
- known: facts directly supported by repo evidence.
- inferred: plausible but not fully proven assumptions.
- open: blockers that cannot be resolved from repo context.

OpenSpec:
- change id and status from openspec list --json, when applicable.
- 相关 OpenSpec artifacts（when applicable）。

Validation:
- Exact commands planned or already run.
- Result, failure state, or why a command could not run.
```

## Assumption Ledger

Use the ledger when a task spans protocol, schema, ref, adapter contract, CLI/MCP output, or handoff boundaries.

```text
ASSUMPTION LEDGER
known:
- Claim: ...
  Evidence: file/ref/command/symbol ...

inferred:
- Claim: ...
  Why it matters: ...
  Evidence needed before edit or before final: ...

open:
- Question: ...
  Blocking condition: ...
```

Before editing, resolve high-risk `inferred` items that affect public behavior. If a low-risk inference remains, state it in the handoff or final answer with the verification that covers it.

## Fresh-Context Handoff

For a fresh-context reviewer or next worker, pass only the packet and the artifact under review. Include:

- The ownership scope and paths.
- The selected role path from `docs/navigation.md`.
- The contract surface being protected.
- The evidence already loaded.
- The exact question the reviewer should answer.

Keep reasoning history out of the handoff unless it is itself evidence. The reviewer should be able to challenge the artifact from sources, not from the previous agent's confidence.
