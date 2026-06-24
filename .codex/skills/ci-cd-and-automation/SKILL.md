---
name: ci-cd-and-automation
description: >-
  设计、修改或调试 contract validation automation、CI validation、workflow automation、
  job ordering、matrix strategy、dependency automation 和 CI failure triage。用于本地工具、
  CLI/API、adapter/service、schema/example、docs 和 Docnav workspace 的验证自动化。
---

# CI/CD 与自动化

## 目标

让自动化对应它声称验证的 contract。每个 job、script、matrix entry 和 required check 都要能回答：它验证哪个 surface，失败时如何本地复现，成功时证明了什么。

## 读取策略

默认只读本文件。Docnav CLI、Rust adapter、schema/example/docs、OpenSpec 或 workspace verifier 相关细节，读 [docnav-contract-validation-automation.md](references/docnav-contract-validation-automation.md)。

Web deployment、browser E2E、database migration、CDN rollout 或 frontend release policy 只有在当前 work item 明确触及这些 surfaces 时才纳入。

## 工作流

1. 识别 changed contract。
   - 命名受影响 surface：CLI/API、adapter/service、schema/example、docs、release/package、dependency 或 workflow behavior。
   - 区分 correctness validation、compatibility validation、security validation、performance budget 和 packaging validation。

2. 选择最小 validation/check set。
   - 优先使用 repository-declared scripts，而不是 inline shell block。
   - 只在 shared behavior、cross-boundary contract 或 declared merge risk 需要时扩大到 workspace-level verifier。
   - Matrix 只覆盖已声明 compatibility risk：runtime version、OS/path behavior、adapter/feature split、packaging target 或 dependency surface。
   - 慢、重复或 observation-only 检查优先作为 report、scheduled job 或手动复现材料；只有明确 budget、public contract 或 merge policy 要求时才成为 required check。

3. 让 failure 保持本地可复现。
   - 记录 command、working directory、tool version、fixture/input、identifier/ref、page、output mode、request payload 和相关 environment variable。
   - 保留能解释 failure 的 machine output、readable output、schema diff、snapshot diff、tool payload、log 或 generated artifact。

4. 先理解 check，再更新 automation。
   - Branch protection 与 required checks 应对应真实 merge risk。
   - Dependency update workflow 使用能证明该 dependency surface 的同一组 required checks。
   - 慢速 compatibility、packaging 或 broad smoke sweep 可以放到 scheduled、merge-required 或 boundary-crossing validation，而不是每个 PR 默认运行。

5. 调试 CI failure。
   - 先在本地重跑完全相同的 command。
   - 将 failure 分类为 setup/environment、compile/type/lint、unit/integration、smoke、schema/docs、packaging 或 dependency。
   - 缩小到仍会失败的最小 fixture、request、ref、page、test name 或 generated artifact。
   - 修复底层 contract；如果 required check 已不符合当前 repository policy，再更新 automation。

## Validation 顺序

默认把快速、确定性的失败放在前面：

1. 从 lockfile install/setup，并安全缓存 Cargo/pnpm。
2. Static check：formatting、linting 和 typechecking。
3. 受影响 package/crate/module 的 build 和 unit tests。
4. 使用稳定 fixture 的 integration/smoke checks。
5. Schema、example、docs 或 change-management validation。
6. Workspace verifier、packaging sweep 或 merge-required compatibility check。

## Matrix 规则

- Runtime: 使用 package 支持的 version range；只有 repository 声明兼容范围时才加入额外版本。
- OS: 只有为 path handling、shell behavior、binary packaging 或 platform-specific failure 提供保护时才加入。
- Feature/adapter/package: 当独立 surface 可独立失败且能缩短 feedback 时拆分。
- Expensive jobs: 如果只重复已证明的 contract，把它留给 merge-required、scheduled 或 boundary-crossing validation。

使用 staged jobs 和 `needs` 让快速 failure 阻止后续昂贵工作。

## 完成检查

结束 CI/automation 工作前确认：

- Commands 使用当前仓库声明的 package manager、toolchain 和 scripts；避免 global install 假设。
- 快速 deterministic check 先于慢速 integration 或 workspace-wide check 运行。
- 每个 changed surface 都有匹配 validation/check，且 required check failure 可本地复现。
- Cross-boundary work 已运行或明确标注需要更宽 verification。
- Failure artifact 足以支持本地复现。
- 最终 diff 保持在预期 automation scope 内。
