# 分离长期决策、OpenSpec 与规范所有权

## 索引摘要
- 目的: 让稳定规则、change 内决策和跨 change 理由各有明确 owner，并保持决策集合可审计。
- 背景: 项目同时使用 owner 文档、OpenSpec change 内决策和长期决策记录，前序记录还固化了具体校验实现。
- 决策: 按作用域分离 owner 文档、OpenSpec change 与长期决策记录，并用仓库内必需校验保护决策集合。

## 目的
- 让稳定规则、变更范围内决策和跨变更长期理由各有唯一 owner，避免相互覆盖、重复维护或被误读为当前实现状态。
- 让全局决策集合随仓库版本化，并由本地与 CI 的必需检查及时发现目录、正文、索引和关系漂移。

## 背景
- 项目同时使用 owner 文档、OpenSpec change 的 `## Decisions` 和全局决策记录；若缺少明确分流，同一判断会被重复维护，change 归档也可能被误解为自动产生长期决策。
- 前序决策正确确立了版本化决策集合和 required 检查，但把直接导入特定模块固化为长期方向；该实现细节应由测试与工具链文档拥有。

## 决策
- 采用: `docs/` owner 文档拥有最终稳定规则，代码、测试和 release artifact 证明当前实现状态。
- 采用: active OpenSpec change 拥有只影响该 change 的目标、设计、决策、任务和验收依据；只有明确确认且跨 change 仍有效的判断才进入 `docs/decisions`，归档 change 不自动复制其中的决策。
- 采用: 长期决策记录保存目的、关键背景、采用方向和演进关系，作为后续工作的默认判断依据，但不覆盖 owner 文档，也不证明当前实现已经支持；失配时必须同步 owner、OpenSpec 和验证材料，并按生命周期处理决策记录。
- 采用: `docs/decisions` 随仓库版本化，并纳入仓库自带、确定性且不依赖个人安装或网络的 required 校验；具体命令、模块和调用路径由测试与工具链文档拥有，可以在保持这些边界时演进。
- 采用: [决策索引](../decision-index.json) 单独管理全局决策生命周期；`openspec/specs/` 只作为 capability specification 的 OpenSpec 工具视图。

## 关系
- 修订: [将仓库决策记录纳入必需验证](use-verified-decision-records.md)
