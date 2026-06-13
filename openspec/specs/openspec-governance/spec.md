# openspec-governance Specification

## Purpose
TBD - created by archiving change normalize-openspec-capability-names. Update Purpose after archive.
## Requirements
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

