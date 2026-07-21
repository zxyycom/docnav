# openspec-governance Specification

## Purpose
定义 OpenSpec 在 Docnav docs-first 工作流中的角色、状态边界和 capability 命名规则，避免 change 规划、长期决策记录、工具视图和主规范互相竞争 owner。
## Requirements
### Requirement: OpenSpec artifacts preserve docs-first ownership
OpenSpec artifacts MUST 支持 Docnav docs-first 工作流。Active change MUST 表示计划中、探索中、实现中或待验收的 change artifact；除非对应主规范明确标注 Current 或已实现，并且代码、测试或 release artifact 提供当前实现证据，否则 active change 中的 `MUST` / `SHALL` MUST NOT 被解释为当前二进制已支持。`openspec/specs/` MAY 作为 capability specification 的 OpenSpec 工具视图；docs、OpenSpec、长期决策记录和实现状态的分工与冲突处理规则由 `docs/navigation.md` 拥有。

#### Scenario: Reading an active change
- **WHEN** 实现者读取 `openspec/changes/<change>/` 下的 proposal、design、tasks 或 spec delta
- **THEN** 该 artifact MUST 被解释为 change 计划、目标或验收依据
- **THEN** 实现者 MUST 使用 `docs/navigation.md` 指向的主规范和当前实现证据判断是否已交付

#### Scenario: OpenSpec and docs conflict
- **WHEN** OpenSpec artifact 与 `docs/` owner 主规范表达不一致
- **THEN** 实现者 MUST 按 `docs/navigation.md` 的冲突类型判断同步方向
- **THEN** 归档前 MUST 让受影响的 docs、OpenSpec artifact、长期决策记录和验证材料回到同一目标决策

### Requirement: Change decisions remain within their scope
Active change 中影响范围、方案、边界或验收的已确认判断 MUST 保存在对应 `design.md` 的 `## Decisions`。该 change 归档时 MUST NOT 自动把这些判断复制到 `docs/decisions/`。判断需要在 change 之外继续作为默认方向，或会改变现有活动决策时，change MUST 按 `docs/navigation.md` 的分工和同步顺序处理后再归档。

#### Scenario: Recording a change-scoped decision
- **WHEN** 已确认判断只约束一个 active change 的方案、边界或验收
- **THEN** 该判断 MUST 写入该 change 的 `design.md` `## Decisions`
- **THEN** 该 change 归档时 MUST NOT 自动把该判断复制到 `docs/decisions/`

#### Scenario: A decision remains relevant beyond the change
- **WHEN** 已确认判断在当前 change 之外仍作为默认方向，或会改变现有活动决策
- **THEN** 归档该 change 前 MUST 按 `docs/navigation.md` 同步 owner 文档和长期决策记录

### Requirement: Capability ID uses stable ownership naming
OpenSpec capability ID MUST 表达长期主 spec 所有权，并 MUST 与一次性 change name 分离。Capability ID MUST 使用 kebab-case 名词或名词短语，MUST NOT 包含 `implement`、`implementation`、`change`、`task`、日期或临时版本阶段。

#### Scenario: Creating a new change
- **WHEN** 实现者为新 OpenSpec change 编写 proposal 的 Capabilities
- **THEN** proposal MUST 明确列出将新增或修改的 capability ID
- **THEN** 每个 capability ID MUST 表达长期能力或稳定责任边界
- **THEN** change name MUST NOT 被默认复用为 capability ID

#### Scenario: Existing capability is affected
- **WHEN** 需求改变已有主 spec 的 requirement
- **THEN** delta spec MUST 使用现有 `openspec/specs/<capability>/spec.md` 的 capability ID
- **THEN** change MUST NOT 创建语义等价的新 capability ID

### Requirement: Capability migration requires an audited mapping
迁移现有 capability ID 前，change artifacts MUST 提供旧 ID 到新 ID 的显式映射、迁移方式和冲突处理。审计通过前，MUST NOT 移动、重命名或修改 `openspec/specs/` 下的现行主 specs。

#### Scenario: Preparing migration artifacts
- **WHEN** change 处于 proposal、design、specs 或 tasks 准备阶段
- **THEN** 迁移映射 MUST 只记录在当前 `openspec/changes/<change>/` 下
- **THEN** 现有 `openspec/specs/` 内容 MUST 保持不变
- **THEN** docs、schema、examples 和实现代码 MUST 保持不变

#### Scenario: Audit approves migration
- **WHEN** 审计确认 proposal、design、specs、tasks 和迁移映射一致
- **THEN** 后续实现任务 MAY 按映射迁移主 specs
- **THEN** 每个旧 capability ID MUST 有明确目标：重命名、合并、拆分或保留

### Requirement: Active changes align before archive
迁移 capability ID 时，所有受影响 active changes MUST 在归档前同步 proposal Capabilities 和 delta spec 目录。未同步的 active change MUST NOT 被归档为旧 capability ID。

#### Scenario: Active change references an old ID
- **WHEN** active change 的 `specs/<capability>/spec.md` 使用旧 capability ID
- **THEN** 迁移任务 MUST 更新该 delta spec 目录到目标 capability ID
- **THEN** 该 change 的 proposal Capabilities MUST 使用相同目标 capability ID

#### Scenario: Archive would recreate old ID
- **WHEN** active change 仍引用旧 capability ID
- **THEN** 归档前检查 MUST 阻止或报告该风险
- **THEN** 实现者 MUST 先完成 active change 对齐或明确延后该 change 的归档
