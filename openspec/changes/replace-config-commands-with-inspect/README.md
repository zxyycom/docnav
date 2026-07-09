# replace-config-commands-with-inspect

Replace legacy docnav config subcommands with a single read-only inspect command backed by owner-provided parameter aggregation metadata, and migrate adapter native option config paths to `options.<adapter-id>.<option-key>`.

Implementation work must keep `config inspect` source-scoped: it reports selected config sources, load state, source summaries, validation diagnostics, and the currently resolvable parameter facts without becoming a config editor or operation dispatch preview.

Typed-fields compound support is conditional: first prove whether existing owner-specific validation covers the current unstructured-full-read array config parity needs; add only the minimum helper if it does not.

## 文档重心

- `README.md`: entry summary, non-negotiable boundaries, and where to read next.
- `proposal.md`: why this change exists, what changes, affected capabilities, and expected impact. It does not define implementation order.
- `design.md`: implementation-shaping decisions, rejected alternatives, risks, complexity, and migration plan. It should explain why the chosen boundaries are acceptable.
- `specs/*/spec.md`: capability-level requirements and scenarios. These files own observable contract deltas, not task sequencing.
- `tasks.md`: execution checklist and phase gates. This file owns the order for moving from proposal to implementation and verification.
