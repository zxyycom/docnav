# Routing Reference

## 读取条件

只在以下情况读取本文件：

1. `SKILL.md` 的快速路由不足以决定 skill 顺序。
2. 用户询问 installed skills、缺失 skills、生命周期顺序或 fallback。
3. 正在维护 `using-agent-skills` 或其它 project skills 的 routing/metadata/reference。

## Installed Skills

| Skill | 使用条件 | 常见组合 |
| --- | --- | --- |
| `interview-me` | 需求缺少 who、why、success、constraint，或用户要求 interview/grill/stress-test | 之后接 `context-engineering` 或 `incremental-implementation` |
| `context-engineering` | agent drift、docs 入口不清、任务切换后上下文风险高 | 之后接具体实现、验证或文档 skill |
| `incremental-implementation` | multi-file change、vertical slice、分阶段验证 | 常接 `api-and-interface-design`；新增或改变 behavior 时接验证 skill |
| `api-and-interface-design` | raw protocol、readable CLI output、adapter contract、ref、pagination、schema、MCP tool mapping | 常接 `doubt-driven-development`，并按 changed surface 选择验证证据 |
| `source-driven-development` | framework/library/API/correctness-sensitive implementation 需要官方资料 | 常接实现或验证 skill |
| `doubt-driven-development` | protocol、schema、ref、adapter contract、CLI/MCP output 等高风险决策需要 bounded challenge | 常接 API 设计和 changed-surface 验证 |
| `test-driven-development` | 新增或改变 behavior、schema/example、adapter protocol、CLI raw/readable、MCP mapping 需要可执行证明 | 常接 debug、implementation、review |
| `debugging-and-error-recovery` | tests fail、build break、可复现异常行为、adapter/CLI/MCP failure 需要 root cause 修复 | 需要新的可执行证明时接 TDD |
| `code-review-and-quality` | PR/local diff/handoff review，关注 correctness、risk 和验证充分性 | 可接 `code-simplification` 或 security/performance |
| `code-simplification` | 在不改行为的前提下降低复杂度、重复或不必要抽象 | 常在 review 后或小重构中使用 |
| `security-and-hardening` | untrusted documents、refs、paths、adapter processes、stdio/JSON、secrets、dependencies | 常接 API 设计和 changed-surface 验证 |
| `performance-optimization` | performance surface、budget 或 profiling 指向 CPU、memory、latency bottleneck | 常接 debugging，并用同一 changed surface 验证优化结果 |
| `ci-cd-and-automation` | declared CLI/Rust/Node/schema/examples/docs contract 需要 automation、workflow、matrix 或 CI failure triage | 常接 docs 或验证 skill |
| `documentation-and-adrs` | ADR、README、CHANGELOG、OpenSpec/docs sync、durable decisions | 常在 public behavior 变化后使用 |

## OpenSpec Skills

OpenSpec skills 只用于 OpenSpec 工作；涉及 `openspec/changes/` 时先按仓库规则运行 `openspec list --json`。

| Skill | 使用条件 |
| --- | --- |
| `openspec-explore` | change 前后澄清想法、调查问题、比较方案或明确边界 |
| `openspec-propose` | 创建可实现的 proposal、design、tasks artifacts |
| `openspec-apply-change` | 推进已接受 change 的 task list |
| `openspec-archive-change` | 实现、验收或同步评估后归档 change |

## 常见组合顺序

完整 Docnav feature 不要求每步都执行；按风险选择最小必要链路。

1. 澄清 intent：`interview-me`
2. 修正上下文：`context-engineering`
3. 查证来源：`source-driven-development`
4. 增量实现：`incremental-implementation`
5. 保护 public contract：`api-and-interface-design`
6. 挑战高风险决策：`doubt-driven-development`
7. 验证 changed surface：`test-driven-development`
8. 修复失败：`debugging-and-error-recovery`
9. 审查风险：`code-review-and-quality`
10. 降低复杂度：`code-simplification`
11. 加固边界：`security-and-hardening`
12. 优化性能：`performance-optimization`
13. 自动化 declared contract：`ci-cd-and-automation`
14. 记录决策：`documentation-and-adrs`

OpenSpec-only 常见组合：`openspec-explore` -> `openspec-propose` -> `openspec-apply-change` -> `openspec-archive-change`。

## 全局行为检查点

这些检查点适用于所有 skill，但不替代具体 skill 的步骤。

1. 假设：在非平凡实现前列出会影响方案的需求、架构、范围假设；若假设不成立会改变结果，先请用户确认。
2. 困惑：发现 spec、代码、用户要求冲突时，说明冲突点、可选解释和推荐路径，再继续。
3. 异议：方案有明确代价时，给出具体风险、可量化影响和替代方案；用户知情坚持后按用户决定执行。
4. 简化：新增抽象前确认它消除了真实重复、稳定了跨模块契约，或降低了长期维护成本。
5. 范围：只改任务要求的文件和行为；相邻清理、顺手重构和删除未知内容需要明确授权。
6. 验证：交付前按 changed surface 运行最小必要验证；已有验证覆盖目标时直接使用它，新增验证只覆盖本次新增或改变的 contract/behavior；无法运行时说明原因和剩余风险。

## Unavailable Recommendations

以下名称可作为“推荐但未安装”的说明，不能当成可调用 workflow：

| 缺失 skill | 正向 fallback |
| --- | --- |
| `idea-refine` | 使用 `interview-me` 或普通澄清问题 |
| `spec-driven-development` | 使用 repository docs、OpenSpec 或 `api-and-interface-design` |
| `planning-and-task-breakdown` | 使用普通 plan 或 `incremental-implementation` |
| `frontend-ui-engineering` | 遵循已有 frontend instructions 和可用 browser tooling |
| `browser-testing-with-devtools` | 使用当前可用 Browser/plugin/tooling；没有工具时说明限制 |
| `git-workflow-and-versioning` | 遵循仓库 git instructions |
| `deprecation-and-migration` | 使用 docs/ADR、implementation、test 和 review skills 组合 |
| `shipping-and-launch` | 使用 `ci-cd-and-automation` 和仓库 release instructions |

## Skill 维护检查

1. `SKILL.md` frontmatter 只保留 `name` 和 `description`。
2. 主文件保留触发条件、最小流程、必要路由和验证边界；长表、生命周期、示例、全局规则放入一层 reference。
3. Reference 必须从 `SKILL.md` 直接链接；避免 reference 再要求深层跳转。
4. 已安装和未安装 skill 分开写；不要把 unavailable recommendation 写成可调用 route。
5. 修改后对改过的 Markdown 运行仓库声明的 Markdown shape/link check；没有专用检查时至少运行 `git diff --check -- .codex/skills`，并检查本 skill 目录内相对链接可解析。
