---
title: 用 Change Plan 承接变更实施并退役 OpenSpec
status: active
alignment: aligned
createdAt: 2026-08-11T02:53:45Z
purpose: 在不重复当前规范和长期方向的前提下，为跨文件 change 保留可交接的实施计划与生命周期。
background: Docs 与决策记录已经分别完整承接当前规范和跨 change 方向，OpenSpec capability 视图继续复制这些内容，但项目仍需要 change 级范围、设计、任务、验证和阶段管理。
decision: Change Plan 承接用户明确要求持久维护的 change-local 计划；OpenSpec 退出当前工作流并只保留历史，其它内容继续由各自 owner 承接。
relations:
  - type: 修订
    target: decision-management/match-change-detail-to-current-phase.md
---

## 目的
- 在不重复当前规范和长期方向的前提下，为跨文件、跨 owner 或跨验证阶段的 change 保留可交接的实施计划与生命周期。
- 让当前事实、未来方向、形成时调查证据和一次 change 的实施上下文各自只有一个明确 owner。

## 背景
- `docs/` owner 文档已经承接 Current 稳定规范，活动决策已经能够自包含地承接确认后的跨 change 方向、理由和约束。
- OpenSpec capability spec 因而与 owner 文档或成熟决策形成重复维护；项目实际仍需要的是 proposal、design、tasks、验证出口、Git 基线和暂停恢复等 change 级协调能力。
- `change-plan` 已提供 draft、plan、implementation、shelved 和 archive 的机械生命周期，并明确不把阶段检查误作内容批准、事实证明或实施授权。
- 调查报告已经有独立的主题、报告快照、资源与索引契约，不应继续夹在 change spec 中充当证据 owner。
- 跨文件、跨 owner 或跨验证阶段说明持久计划可能有价值，但不等于用户已经授权 agent 新建持久工作制品。

## 决策
- 采用: `docs/` owner 文档继续拥有 Current 稳定规范；`docs/decisions/` 继续拥有已经确认、跨 change 有长期影响的方向与理由，不再为这些内容维护 OpenSpec capability 副本。
- 采用: `changes/<change>/` 的 Change Plan 拥有一次 change 的目标、范围、change-local 设计、任务、验证和阶段；创建与写入授权由项目级 `change-plan` skill 精确规定，Docnav 不增加更宽例外。复杂工作可能需要计划时，agent 只提醒或建议创建，不自行落盘。
- 采用: 已确认 Plan 可以包含先于代码的 change-local 人工批准、证据或依赖门禁；只要目标、设计、门禁 owner、关闭动作、被阻塞任务和验证顺序已经完整，存在这些门禁不把计划降回只保存方向的 Draft，也不授权绕过门禁实施。
- 采用: Change Plan 的 status、stage、assessment 和任务勾选只表达计划机械状态，不证明方案获批、实现完成或行为已经成为 Current，也不扩大当前任务授权。
- 采用: `docs/investigations/` 继续保存形成时调查证据与认识快照；调查形成的长期方向、实施任务或稳定事实分别交给决策、Change Plan 或 owner 文档。
- 采用: 既有 OpenSpec 内容整体移入 `archive/legacy/openspec/`，只保留为切换时历史和审计材料，包括当时未完成的 change；OpenSpec CLI 状态、capability spec 和未完成目录不再作为当前计划、未来方向或稳定规范。
- 采用: 项目验证使用仓库跟踪的 `change-plan`、`decision-records` 和 `investigation-report` CLI 检查当前集合，不在日常或 CI 验证中依赖个人安装、联网更新或 OpenSpec 工具。
