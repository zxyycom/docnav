# Docnav Review Cues

## 适用范围

只在审查 Docnav contract、CLI、adapter、ref、pagination、raw/readable output、schema/example、MCP bridge、docs 或 OpenSpec artifacts 时读取本 reference。普通代码审查先使用 `../SKILL.md`。

## Contract Cues

- 保持 CLI-first navigation flow: `outline -> ref -> read`.
- Treat refs as adapter-generated and adapter-owned. Core `docnav`, MCP and other entry points pass them through; they do not parse, rewrite or invent refs.
- Separate raw protocol output from readable output. They can share business semantics, but not transport wrappers, schema, pagination envelope or stability promises.
- Check pagination and continuation: bounded reads, stable metadata, enforced limits, deterministic ordering, and no accidental full-document load.
- Keep adapter responsibilities inside adapters: format detection, parsing, navigation strategy, ref generation/parsing, pagination and direct adapter CLI behavior.
- Keep the MCP bridge thin. It maps tool calls to `docnav` behavior and does not copy adapter parsing, routing or protocol logic.
- Check Windows path behavior when CLI or process boundaries move: drive letters, backslashes, spaces, quoting, stdin/stdout/stderr and readable error output.
- When protocol, schema, examples, CLI output, adapter behavior or MCP mapping changes, verify the governing docs and validation artifacts are updated.
- If the work item has OpenSpec artifacts, review consistency across artifacts, implementation, tests and verification.

## Test Layer Cues

- Adapter behavior: adapter unit/smoke tests and focused fixtures.
- Core CLI/routing/config/output mode: CLI integration or smoke tests.
- MCP mapping: bridge tests that compare tool args/results with owning CLI behavior.
- Schema/example changes: schema validation, fixture/example round trip, generated output diff.
- Bug fixes: regression proof that fails before the fix and passes after it.

## Verification Scope

Use repository-declared commands and current docs rather than hardcoded build output paths.

- Markdown-only skill/reference changes: run available Markdown shape/link checks, then `git diff --check -- .codex/skills`.
- Adapter or direct CLI behavior: run the relevant adapter smoke/integration checks.
- Core CLI/routing/config: run the relevant core CLI smoke/integration checks.
- Protocol, schema, examples, docs, MCP mapping or cross-boundary work: run the repository workspace verifier when feasible, or record the narrow checks and skipped wider gate.
