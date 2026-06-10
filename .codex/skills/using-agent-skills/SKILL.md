---
name: using-agent-skills
description: 在 Docnav 中选择和维护已安装的 project skills；仅用于 skill routing 含糊、维护 skill route/metadata/reference，或用户询问 installed skill 适用性时使用，不作为每次任务的默认入口。
---

# Using Agent Skills

## 定位

这是 Docnav project skills 的 routing navigator。只有在以下情况触发：

1. 用户询问“该用哪个 skill”、installed skills、skill routing 或 skill 维护。
2. 任务明显需要 skill，但多个 installed skills 都可能适用，优先级不清。
3. 需要更新本仓库 `.codex/skills/` 下的 project skill 说明、路由、reference 或 metadata。

如果用户请求已经直接命中某个 installed skill，直接读取那个 skill；不要先经过本 skill。

## 最小流程

1. 判定任务类型：routing 咨询、skill 维护、普通实现、OpenSpec、review、debug、docs、CI、安全、性能或测试。
2. 选择最小 skill 集：只选能改变执行路径的 skill；多个 skill 同时适用时说明使用顺序。
3. 读取目标 skill 的 `SKILL.md`，并只按其说明加载必要 reference。
4. 如果所需 skill 未安装，明确说明 unavailable，再使用仓库规则或最接近的 installed workflow 继续。
5. 完成前检查：路由是否来自 installed skills、是否遗漏验证步骤、是否把本 skill 当成默认入口。

## Reference 读取

需要完整 route map、常见使用顺序、全局行为检查点、未安装 skill fallback，或正在维护本 skill 时，读取 [routing-reference.md](references/routing-reference.md)。

`references/original-skill.md` 是历史备份；只在对照原始文本时读取，不在常规 routing 中加载。

## 快速路由

| 任务信号 | 读取的 installed skill |
| --- | --- |
| 需求、成功标准或约束不清 | `interview-me` |
| context drift、误读 docs、规则冲突 | `context-engineering` |
| multi-file change、分片交付、实现推进 | `incremental-implementation` |
| protocol、schema、ref、CLI/MCP contract | `api-and-interface-design` |
| 需要 official docs 或 authoritative source | `source-driven-development` |
| 高风险 protocol/ref/schema 决策 | `doubt-driven-development` |
| tests、fixtures、行为证明 | `test-driven-development` |
| failure、regression、异常行为 | `debugging-and-error-recovery` |
| pre-merge review、quality gate、风险审查 | `code-review-and-quality` |
| 保持行为不变地减复杂度 | `code-simplification` |
| untrusted docs、paths、refs、stdio、secrets | `security-and-hardening` |
| 已测量的 performance regression 或 budget | `performance-optimization` |
| CI、validation、automation pipeline | `ci-cd-and-automation` |
| docs、ADR、durable decisions | `documentation-and-adrs` |
| OpenSpec change 探索、提案、实现、归档 | `openspec-explore` / `openspec-propose` / `openspec-apply-change` / `openspec-archive-change` |

## 组合规则

1. 常规实现：`incremental-implementation` 可与 `test-driven-development`、`api-and-interface-design` 或 `source-driven-development` 组合。
2. 缺陷修复：优先 `debugging-and-error-recovery`，需要行为锁定时加 `test-driven-development`。
3. 高风险公共契约：`api-and-interface-design` 后接 `doubt-driven-development`，再用 tests 或 docs 验证。
4. OpenSpec 工作：只在任务确实涉及 OpenSpec change 时使用 `openspec-*` skills，并遵循仓库 OpenSpec 读取规则。
5. Skill 维护：同时遵循 `skill-creator`；若是在优化规则文本，同时遵循 `prompt-optimize`。

## 边界

- Installed skill 才能作为可调用 workflow；缺失 skill 只能作为 unavailable recommendation。
- Skills 是执行流程，不是标签；一旦选用，就按对应 skill 的步骤和验证要求执行。
- 本 skill 只解决 routing 和维护问题；具体实现、测试、审查或文档工作交给被选中的 skill。
- 安全、权限、事实真实性和不可逆操作边界按仓库规则优先；给出正向做法，再继续任务。
