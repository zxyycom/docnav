---
name: documentation-and-adrs
description: >-
  记录 durable decisions 和可维护文档上下文。用于 architectural decisions、public API/contract
  变更、功能交付文档、ADR、README、CHANGELOG、OpenSpec/change docs、Docnav docs sync，
  以及为未来工程师和 agent 留下可验证上下文。
---

# 文档与 ADR

## 目标

为未来工程师和 agent 留下能继续决策的上下文。文档优先回答 why、约束、取舍、变更影响和验收方式；代码和类型负责表达 what。ADR 只记录长期架构决策的 durable rationale；change-local rationale 留在 OpenSpec design 或相关 docs。

## 触发后流程

1. 定位文档职责：
   - 长期架构方向、跨多个 change 的选择、回滚或迁移成本高、未来会反复争论的 tradeoff：写 ADR。
   - 单个 change 的方案依据、task breakdown、spec delta 或 acceptance：留在 OpenSpec design、tasks 或相关主规范。
   - 使用、运行、贡献或发布信息：更新 README、CHANGELOG 或相邻说明。
   - 只解释局部非显而易见约束：写 inline documentation。

2. 读取项目上下文：
   - 先读取 repository rules、navigation docs 或 owner docs；只读取当前文档任务需要的 authoritative source。
   - Change-management artifacts 只在处理 change、验收、历史审计或用户明确要求时读取。
   - Schemas、examples 和 generated fixtures 是验证材料，只在字段、示例、schema、machine output 或测试相关时读取。

3. 选择 reference：
   - ADR 触发、轻量模板或历史链接：读 [adr-guide.md](references/adr-guide.md)。
   - inline docs、API docs、README、CHANGELOG 或 agent-facing docs：读 [documentation-patterns.md](references/documentation-patterns.md)。
   - Docnav contract surface 同步：读 [docnav-docs-sync.md](references/docnav-docs-sync.md)。

4. 编写或修改：
   - 把模糊要求改成可检查的完成条件。
   - 删除重复、过期或只复述代码的内容。
   - 保留历史 rationale；长期决策变化时新增 ADR 或链接说明。
   - 不把 ADR 写成 proposal、task breakdown、spec delta 或 acceptance 流程。
   - 让文档链接到权威来源、相关 ADR、schema、examples 或 tests，而不是复制整套规范。

5. 交付前自检：
   - 文档是否解释了 why、约束、取舍和影响。
   - 对应 contract surface 是否已同步主规范、schema、examples 和测试材料。
   - 相对链接是否可解析。
   - README/CHANGELOG/API docs 是否只覆盖当前需要维护的事实。

## 判断标准

- ADR：记录长期架构决策、rationale、后果和链接；适合跨 change 影响、回滚成本高或会反复争论的选择。
- Inline docs：解释局部 gotcha、非显而易见约束或与 ADR 的连接；不解释自解释代码。
- API docs：说明输入、输出、错误、稳定性和示例；类型定义是第一层文档。
- Agent docs：记录仓库工作方式和上下文入口；不要复制完整产品规范。
- CHANGELOG：记录已交付、用户可见或集成方可见的变化。

## Docnav 定制边界

Docnav 仓库内的文档入口、同步矩阵、OpenSpec 使用边界、schema/examples 验收标准见 [docnav-docs-sync.md](references/docnav-docs-sync.md)。只有当前文档任务触及 Docnav contract surface 时读取该 reference。

## 验收

完成文档任务后确认：

1. 变更范围与用户请求一致，没有顺手重写无关文档。
2. 所有新增长期规则都有明确触发条件和验收方式。
3. 长期架构决策有 ADR 或链接到现有 ADR；change-local rationale 留在 OpenSpec design 或相关 docs。
4. public contract 变化已同步主规范、schema、examples 和 tests。
5. 相对链接、标题和引用目标可解析。
