---
name: ci-cd-and-automation
description: "自动化 Docnav quality gate 与 CI validation。用于设置或修改 Docnav CLI、Rust adapter、Node.js MCP bridge、schema、example、docs、OpenSpec change、workflow automation、CI job ordering、matrix strategy，以及 Rust + Node workspace 中的 CI failure triage。"
---

# CI/CD 与自动化

## 触发场景

当需要修改、审查或调试以下 Docnav quality gate 时使用本 skill：

- `docnav` CLI behavior、output mode、error mapping、config、init 或 adapter routing。
- Rust adapter，包括 Markdown smoke coverage，以及后续 JSON/YAML/TOML/INI gate。
- Node.js MCP bridge 的 package check，或 tool-call 到 `docnav` 的 mapping。
- Schema、example、docs、OpenSpec change、release/build/package check、branch protection 或 dependency automation。
- CI job ordering、matrix selection、cache、artifact，以及 CI failure feedback loop。

## 非触发场景

不要默认围绕通用 web-app deployment 优化。除非某个 Docnav task 明确加入这些 surface，否则 Vercel preview、Prisma/Postgres migration、browser E2E、CDN rollout 和 frontend release gate 都不在范围内。

不要仅因为存在 CI 就加入宽 matrix 或昂贵的 workspace verifier。每个 job 都要绑定到它保护的 Docnav contract。

## 工作流

1. 识别 changed contract。
   - CLI 与 core routing：验证 `outline -> ref -> read`、format detection、output mode、config/default、error mapping。
   - Rust adapter：验证 probe behavior、parsing、navigation、ref 生成/解析、pagination、直接 adapter CLI output 和 error path。
   - Node MCP bridge：验证 MCP tool payload 映射到 `docnav` invocation，且没有复制 parser 或 router logic。
   - Schema/example/docs：用 repository script 校验 raw protocol field、example 和 docs。
   - OpenSpec：先运行 `openspec list --json`，再用 `openspec validate` 校验相关 change。

2. 选择最小且可复现的 gate set。
   - 确认 script 存在后，优先使用已有 `pnpm run <script>` check。
   - Node dependency 使用 `pnpm install --frozen-lockfile`。
   - Rust 使用 Cargo：`cargo fmt --check`、repository clippy policy、定向 `cargo test -p <crate>`；当共享行为变化时使用 `cargo test --workspace`。
   - 对代表性的 `info`、`outline`、`read`、pagination 和 error-path request，使用 adapter smoke script 或 direct binary。
   - 如果引入新的 Python script dependency，使用 `uv`。
   - Cross-boundary change 最后运行 `pnpm run verify:docnav-workspace`。

3. 让 failure 保持本地可复现。
   - 记录 command、working directory、tool version、fixture path、ref、page、output mode 和相关 environment variable。
   - 当 protocol JSON/readable output、schema diff、snapshot diff、MCP payload、log 或 generated artifact 能解释 failure 时，保留它们。

4. 先理解 gate，再更新 automation。
   - 优先使用 repository script，而不是 inline shell block。
   - 让 branch protection 与能证明 Docnav contract 的 check 保持一致。
   - 只有当 dependency update workflow 运行与普通 PR 相同的 gate 时，才使用它。
   - 慢速 compatibility 或 packaging sweep 应改为定时运行，而不是把每个昂贵 job 都放进每个 PR。

## Gate 映射（Gate Map）

- CLI/core：受影响 crate test、binary build，以及覆盖 `info`、`outline`、`read`、output mode 和 error mapping 的 CLI smoke check。
- Rust adapter：adapter crate test、基于 fixture 的直接 adapter CLI smoke check、protocol-json verification、pagination test 和 ref parsing test。
- Node MCP bridge：`pnpm` install、package lint/type/test script、tool-call mapping test，以及调用 `docnav` 的 bridge smoke check。
- Schema/example：schema validation script、fixture/example round trip，以及 generated output diff check。
- Docs：docs validation script，以及反映当前 CLI 行为的 example command。
- OpenSpec：`openspec list --json`、带 repository-required flag 的 `openspec validate <change>`，以及 task checklist review。
- Cross-boundary change：`pnpm run verify:docnav-workspace`。

## CI 顺序与矩阵（Matrix）

默认顺序：

1. 从 lockfile install/setup，并安全缓存 Cargo/pnpm。
2. Static check：formatting、linting 和 typechecking。
3. 受影响 Rust crate 与 Node package 的 build 和 unit test。
4. 使用稳定 fixture 的 CLI 与 adapter smoke check。
5. Schema、example、docs 和 OpenSpec validation。
6. Workspace verifier 或必需的 merge gate。

只有当 matrix 保护已声明的 compatibility contract 时才使用：

- Rust：默认 stable；只有 repository 声明 MSRV 时才加入 MSRV。
- Node.js：使用 package 支持的 version range，不随意使用 latest。
- OS：只有为 path handling、shell behavior、binary packaging 或 platform-specific failure 提供保护时，才加入 Windows/macOS。
- Adapter/feature：当独立 adapter/protocol surface 可独立失败时，拆分 job。

使用带 `needs` 的 staged job，让快速确定性 failure 阻止后续工作。如果 `verify:docnav-workspace` 与早期 gate 重复，保留窄 job 提供快速反馈，并把 verifier 留给 merge-required 或 boundary-crossing validation。

## 失败分诊（Failure Triage）

CI 失败时：

1. 改代码前，先在本地重跑完全相同的 command。
2. 将 failure 分类为 environment/setup、Rust compile/test、Node package、CLI/adapter smoke、schema/example/docs、OpenSpec 或 MCP bridge mapping。
3. 缩小到仍会失败的最小 fixture、ref、page、request payload 或 test name。
4. 修复底层 contract；如果 gate 已不符合 repository policy，则更新 gate。
5. 先重跑窄的 failed gate，再重跑原本应捕获该 failure 的更宽 gate。

## 交付前验证

结束 CI/automation 工作前确认：

- Command 使用 `pnpm`、Cargo；如引入 Python script，则使用 `uv`；优先 repository script，而不是 global install。
- 快速 deterministic check 先于慢速 integration 或 workspace-wide check 运行。
- 每个 changed surface 都有匹配 gate：CLI、Rust adapter、Node MCP bridge、schema、example、docs 或 OpenSpec。
- Cross-boundary work 已运行或明确需要 `pnpm run verify:docnav-workspace`。
- Failure artifact 足以支持本地复现。
- 最终 diff 保持在预期 automation scope 内。
