# support-jsonc-in-json-adapter

These temporary planning artifacts define one controlled JSONC-capable grammar for the built-in JSON adapter while preserving strict-JSON document semantics, auditable source mapping, normalized structured output, and bounded compatibility with adjacent JSON families.

## Goal

Extend the existing `docnav-json` strategy, format identity, and operation surface to accept strict JSON plus the closed JSONC grammar selected in [design.md](design.md). The change keeps source output faithful to the JSONC input and keeps structured output normalized as JSON.

## Boundary

This change owns JSONC grammar, parsing, navigation, source mapping, normalized output, diagnostics, and the `.jsonc` routing hint plus `application/jsonc` descriptor content type. It does not add profile semantics, remote resolution, JSON5, multiple JSON roots, a public dialect/mode abstraction, later JSON-family pathname hints, or a binary JSON-family model.

## Sequencing and status

The observable contract in Decisions 1–10 is selected, and the manifest-native routing predecessor is already Current. Parser implementation and dependency selection remain unapproved: tasks 0.1–0.10 must produce the required evidence, task 0.8 requires an authorized approver, and task 0.11 must close the blocking audit before any production, owner-doc, schema/example, fixture, test, dependency, or release work begins. `expand-json-adapter-pathname-hints` is a downstream change and does not block this change or own its parser semantics.

## Reading path

Read [proposal.md](proposal.md) for the target outcome, [design.md](design.md) for selected decisions and remaining gates, [specs/json-adapter/spec.md](specs/json-adapter/spec.md) for the complete target delta, and [tasks.md](tasks.md) for the executable sequence. Begin with blocking task 0; OpenSpec artifact completeness alone does not make this change implementation-ready.
