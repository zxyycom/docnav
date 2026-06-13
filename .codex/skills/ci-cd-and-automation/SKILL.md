---
name: ci-cd-and-automation
description: >-
  设计、修改或调试 contract-driven quality gates、CI validation、workflow automation、
  job ordering、matrix strategy、dependency automation 和 CI failure triage。用于本地工具、
  CLI/API、adapter/service、bridge、schema/example、docs 和 Docnav Rust+Node workspace 的验证自动化。
---

# CI/CD 与自动化

## 目标

让自动化证明它声称保护的 contract。每个 job、script、matrix entry 和 merge gate 都要能回答：它保护哪个 surface，失败时如何本地复现，成功时证明了什么。

## 读取策略

默认只读本文件。Docnav CLI、Rust adapter、Node MCP bridge、schema/example/docs、OpenSpec 或 workspace verifier 相关细节，读 [docnav-quality-gates.md](references/docnav-quality-gates.md)。

Web deployment、browser E2E、database migration、CDN rollout 或 frontend release gate 只有在当前 work item 明确触及这些 surfaces 时才纳入。

## 工作流

1. 识别 changed contract。
   - 命名受影响 surface：CLI/API、adapter/service、bridge/tool mapping、schema/example、docs、release/package、dependency 或 workflow behavior。
   - 区分 correctness gate、compatibility gate、security gate、performance budget 和 packaging gate。

2. 选择最小 gate set。
   - 优先使用 repository-declared scripts，而不是 inline shell block。
   - 只在 shared behavior 或 merge policy 需要时扩大到 workspace-level verifier。
   - Matrix 只覆盖已声明 compatibility risk：runtime version、OS/path behavior、adapter/feature split、packaging target 或 dependency surface。

3. 让 failure 保持本地可复现。
   - 记录 command、working directory、tool version、fixture/input、identifier/ref、page、output mode、request payload 和相关 environment variable。
   - 保留能解释 failure 的 machine output、readable output、schema diff、snapshot diff、tool payload、log 或 generated artifact。

4. 先理解 gate，再更新 automation。
   - Branch protection 与 required checks 应对应真实 merge risk。
   - 只有当 dependency update workflow 运行与普通 PR 相同的 gate 时，才使用它。
   - 慢速 compatibility、packaging 或 broad smoke sweep 可以放到 scheduled/merge gate，而不是每个 PR 默认运行。

5. 调试 CI failure。
   - 先在本地重跑完全相同的 command。
   - 将 failure 分类为 setup/environment、compile/type/lint、unit/integration、smoke、schema/docs、bridge/tool mapping、packaging 或 dependency。
   - 缩小到仍会失败的最小 fixture、request、ref、page、test name 或 generated artifact。
   - 修复底层 contract；如果 gate 已不符合当前 repository policy，再更新 gate。

## Gate 顺序

默认把快速、确定性的失败放在前面：

1. 从 lockfile install/setup，并安全缓存 Cargo/pnpm。
2. Static check：formatting、linting 和 typechecking。
3. 受影响 package/crate/module 的 build 和 unit tests。
4. 使用稳定 fixture 的 integration/smoke checks。
5. Schema、example、docs 或 change-management validation。
6. Workspace verifier、packaging sweep 或 merge-required compatibility gate。

## Matrix 规则

- Runtime: 使用 package 支持的 version range；只有 repository 声明兼容范围时才加入额外版本。
- OS: 只有为 path handling、shell behavior、binary packaging 或 platform-specific failure 提供保护时才加入。
- Feature/adapter/package: 当独立 surface 可独立失败且能缩短 feedback 时拆分。
- Expensive jobs: 如果与早期 gate 重复，把它留给 merge-required、scheduled 或 boundary-crossing validation。

使用 staged jobs 和 `needs` 让快速 failure 阻止后续昂贵工作。

## 完成检查

结束 CI/automation 工作前确认：

- Commands 使用当前仓库声明的 package manager、toolchain 和 scripts；避免 global install 假设。
- 快速 deterministic check 先于慢速 integration 或 workspace-wide check 运行。
- 每个 changed surface 都有匹配 gate，且 gate failure 可本地复现。
- Cross-boundary work 已运行或明确标注需要更宽 verification。
- Failure artifact 足以支持本地复现。
- 最终 diff 保持在预期 automation scope 内。
