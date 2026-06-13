# Docnav Contract Scope

## 适用范围

只在 Docnav 仓库内处理 public contract、CLI-first navigation、adapter ownership、ref、pagination、schema/example、error mapping 或 MCP bridge mapping 时读取本 reference。通用 REST、GraphQL 或 TypeScript API 设计不使用本文件。

## Contract Surfaces

- Raw protocol: machine-readable request/response fields、envelope、error shape、pagination metadata 和 adapter-generated ref。
- Readable CLI output: text/readable JSON 的 ordering、label、信息密度、truncation、continuation hint 和人类可读 error text。
- CLI surface: top-level command、flag、default、output mode、exit behavior、project/config behavior。
- Adapter contract: format detection、parsing ownership、outline/read/find/info semantics、direct adapter CLI output、adapter-owned ref generation/parsing。
- Ref: adapter-owned opaque identifier；core、MCP 和其它入口只原样传递。
- Pagination/continuation: page number、limit、truncation indicator、稳定排序、resumable next-step instruction。
- Schema/example: JSON schema、example fixture、golden output 和 docs snippet，用于校验或展示 contract。
- Error mapping: adapter/core error 到 protocol code、CLI exit/readable message、MCP tool error 的一致映射。
- MCP tool mapping: stdio bridge 的 tool name、input schema、output mapping，以及到 `docnav` behavior 的 pass-through。

## Ownership Rules

1. `docnav` core owns format routing、adapter management、shared CLI defaults、output mode、configuration、project init 和 error mapping。
2. `docnav-mcp` maps MCP tool calls to `docnav`; it does not duplicate parsing、routing 或 adapter logic。
3. Format adapters own format detection、parsing、navigation strategy、ref generation/parsing、pagination result 和 direct adapter CLI behavior。
4. Raw protocol and readable output share business semantics, not transport wrappers.
5. Schema、examples、fixtures、docs 和 tests that validate or display an observable change must be updated with the same work item.

## Verification Scope

Choose the smallest repository-declared checks that prove the changed surface. Prefer commands from `package.json`, repository docs, nearby tests, or current AGENTS instructions. Do not hardcode build output paths in reusable rules.

- Skill or Markdown-only changes: run repository Markdown shape/link checks when available, then `git diff --check -- .codex/skills`.
- Schema/example changes: run the validation command documented beside the artifact.
- CLI、adapter、MCP、schema/example 或 docs boundary changes: run the relevant smoke/integration checks; for cross-boundary Docnav work, use the repository workspace verifier when feasible.
- Always review the local diff and confirm the changed files match the intended ownership scope.
