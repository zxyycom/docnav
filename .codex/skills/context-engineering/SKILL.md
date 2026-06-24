---
name: context-engineering
description: >-
  建立、修复或交接 project context。用于 agent drift correction、context-risk task switches、
  stale/conflicting assumptions recovery、parallel worker handoff，以及维护 AGENTS/rules files、
  project skills 或 Docnav repository context。
---

# 上下文工程（Context Engineering）

## 用途

使用本技能建立、修复或交接 project context。目标是让 agent 用最小但可追溯的 context packet 锚定仓库契约、编辑范围、authority path、assumptions 和验证路径。

这不是 session startup checklist。只有当任务本身涉及 context quality、context drift、handoff、rules files、project skills，或跨 public contract 边界时才使用。

## 触发条件

- Agent 行为偏离 repository rules、architecture boundaries、文档读取规则或用户 scope。
- 任务跨越多个 ownership boundaries，例如 CLI/API、adapter/service、protocol、schema、docs、examples 或 change-management artifacts。
- compaction、handoff、长线程或 parallel workers 需要一个可继续的 context packet。
- 需要创建、修复、合并或 review `AGENTS.md`、project skills 或其它 rules files。
- 当前 assumptions 与 files、refs、CodeGraph、tests 或 command output 出现冲突。

## 最小流程

1. **界定 scope**: 写清 task owner、允许编辑路径、并行协作边界和本轮不能触碰的文件。
2. **选择 authority path**: 从仓库 rules、navigation docs、owner docs 或 governing spec 选择当前任务的 authoritative source，只读取相关部分。
3. **建立 evidence gate**: 每个会影响行为、contract 或编辑范围的 assumption，都必须绑定 source：rules file、bounded doc read、CodeGraph symbol、filtered search、test 或 command output。
4. **维护 assumption ledger**: 标记 `known`、`inferred`、`open`。高风险 `inferred` 在编辑前补证据或缩小 scope。
5. **打包 context**: handoff、drift recovery 或高风险改动时，用 [context-packet.md](references/context-packet.md) 输出紧凑 packet。
6. **修复 drift**: 当事实冲突或 context 过载时，按 [drift-recovery.md](references/drift-recovery.md) 重新定位 authoritative source。

## 读取策略

按顺序加载，满足当前 task 后停止：

1. `AGENTS.md` 和贴近编辑路径的 rules files，作为 repository work contract。
2. Repository navigation docs 或 owner docs；在 Docnav 内从 `docs/navigation.md` 的 role path 进入。
3. CodeGraph：`codegraph_search` 定位，`codegraph_node` 先看签名和位置，需要源码时再 `includeCode=true`；用 callers/callees/impact 判断 refactor risk。
4. Markdown 或大型文档：使用仓库声明的 bounded navigation command；在 Docnav 内遵循 AGENTS 中的 `outline -> ref -> read` 规则。
5. OpenSpec/change-management artifacts：只在相关 work、historical audit、validation 或用户明确要求时读取；按仓库规则先列出现有 changes。
6. Schemas/examples/generated fixtures：只在验证 fields、output shape、examples 或 tests 时读取。
7. Verification output：只纳入 command、聚焦 error lines、affected files 和当前 failure state。

## Docnav Scope

Docnav 专属读取规则、architecture boundaries、drift signals 和 rules-file 检查点在 [drift-recovery.md](references/drift-recovery.md)。需要 handoff packet 或 assumption ledger 时读 [context-packet.md](references/context-packet.md)。

## 维护 Rules Files

创建或修复 `AGENTS.md`、project skills 或其它 rules files 时，优先把规则写成正向检查点：

- scope、ownership、parallel worker safety 和允许编辑路径明确。
- 文档入口指向项目 navigation 或 owner docs，详细 specs 只作为 role path 后的读取结果。
- Tool rules 包含 preferred code-structure tool、bounded Markdown reads、change-management reads only when relevant、schemas/examples only for validation。
- 长模板、failure modes、review prompts 和 checklists 放入直接链接的 reference。
- 环境中不可用的 services、tool names、generic framework examples 不进入 rules。

## References

- [context-packet.md](references/context-packet.md): context packet、assumption ledger、fresh-context handoff 模板。
- [drift-recovery.md](references/drift-recovery.md): context drift signals、evidence gate、rules file review checklist。

## 验证

完成前确认：

- Trigger 是 task-specific，不是默认 session bootstrap。
- 选定 authority path 来自 repository rules、navigation docs、owner docs 或 governing spec。
- 大型 Markdown/doc reads 使用仓库声明的 bounded navigation path；没有可用命令时才回退普通读取。
- CodeGraph 或 filtered search 已覆盖相关 implementation surface。
- Change-management artifacts、schemas、examples 只在相关时被纳入。
- Context packet 或 rules update 写明 precise verification command 和结果。
