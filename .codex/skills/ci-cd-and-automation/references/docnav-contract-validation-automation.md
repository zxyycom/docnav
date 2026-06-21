# Docnav Contract Validation Automation

## 适用范围

只在 Docnav 仓库内设置、修改或调试 CLI、Rust adapter、Node MCP bridge、schema、examples、docs、OpenSpec 或 workspace verification 时读取本 reference。这里的 validation map 只描述 Docnav surfaces 的最小自动化验证入口，不迁移成通用 CI 规则。

## Validation Map

- CLI/core: affected crate tests、binary build，以及覆盖 `info`、`outline`、`read`、output mode 和 error mapping 的 CLI smoke check。
- Rust adapter: adapter crate tests、fixture-based direct adapter smoke、protocol JSON verification、pagination tests 和 ref parsing tests。
- Node MCP bridge: package install/check/test、tool-call mapping tests，以及 bridge-to-`docnav` smoke checks。
- Schema/example: schema validation、fixture/example round trip 和 generated output diff check。
- Docs: docs validation、links/shape checks，以及反映当前 CLI behavior 的 examples。
- OpenSpec: 只在 OpenSpec work、audit、validation 或用户明确要求时运行 `openspec list --json`，再校验相关 change。
- Cross-boundary change: when CLI/core、adapter、schema/example、docs 或 MCP mapping move together, run the repository workspace verifier when feasible.

## Repository Command Policy

1. Prefer scripts already declared in the repository. Confirm a script exists before requiring it.
2. For Node/JavaScript work, use the repository package manager and lockfile policy.
3. For Rust work, use repository Cargo formatting, lint and test policy; target affected packages before workspace-wide checks.
4. For Python helper scripts, use the repository-approved Python runner.
5. Do not hardcode build output binary paths in reusable skill rules. If a direct adapter CLI replay is needed, resolve it from current docs, scripts, or the build artifact produced in this task.

## Failure Triage

1. Re-run the exact failing command locally before changing code or workflow YAML.
2. Classify failure: environment/setup、Rust compile/test、Node package、CLI/adapter smoke、schema/example/docs、OpenSpec 或 MCP bridge mapping。
3. Shrink to the smallest failing fixture、ref、page、request payload、generated artifact 或 test name。
4. Fix the underlying contract first; update the check only when it no longer matches repository policy.
5. Re-run the narrow failed check, then the wider verification that proves the declared merge risk.

## Workspace Verification Trigger

Use the repository workspace verifier, or record why it was skipped, when a work item crosses Rust crates, CLI/core plus adapter, schema/examples, docs that publish command behavior, generated fixtures, MCP tool mapping, or stdio/JSON smoke coverage.
