---
title: 通用代码质量规则由编码规范统一拥有
status: active
alignment: aligned
createdAt: 2026-08-12T03:42:03Z
purpose: 让实现与审查从固定 owner 路径恢复通用代码质量要求，避免项目 Skill 重复编码规则。
background: 编码规范已覆盖通用实现质量，按工程动作拆分的 Skill 会重复 owner、扩大触发范围并增加同步责任。
decision: 行为 owner 先定义必须做什么，docs/coding-style.md 再统一约束如何实现；AGENTS.md、文档导航和 Skill 只负责路由。
relations: []
---

## 目的

- 让实现者、AI coding agent 和 reviewer 使用同一读取顺序，准确区分产品行为与通用实现质量。
- 让行为 owner、编码规范和 agent 路由各自只完整承接一类规则，减少重复、触发噪声和同步负担。

## 背景

- `docs/coding-style.md` 已经拥有实现归属、最小正确模型、边界与失败、类型表达、模块组织、稳定规则归位和风险验证等通用要求。
- `AGENTS.md` 和 `docs/navigation.md` 已经能够把相关任务引导到行为 owner、编码规范和验证入口；路由位置不需要复制完整规则。
- 通用编码 Skill 曾重复这些要求并嵌入 output mode、ref、pagination、adapter 等 Current 项目术语。产品契约演进时，非 owner Skill 也需要同步修改。
- 多个宽触发编码 Skill 会在同一实现、调试或审查任务中叠加上下文，却不提供新的项目事实、工具或持久流程。

## 决策

- 采用: 实现、重构和涉及实现代码的审查先从 `docs/navigation.md` 指向的行为 owner 恢复“必须做什么、由谁负责、如何观察”，再使用 `docs/coding-style.md` 约束实现选择与验收。
- 采用: `docs/coding-style.md` 完整拥有跨组件通用的实现代码质量规则；相邻代码只提供当前实现证据，不能覆盖行为 owner 或编码规范。
- 采用: `AGENTS.md`、`docs/navigation.md` 和保留的项目 Skill 只声明读取时机、责任路由或必要摘要，不重新完整表达编码规范。
- 边界: 编码规范不拥有产品行为、协议字段、组件职责、测试覆盖语义或工具命令；这些内容继续由各自 owner 完整承接。
- 不采用: 仅为了强化普通编码质量而建立内容等价的 `coding-style` Skill，或把编码规范拆成多个按工程动作命名的 Skill。若 agent 无法取得 owner 文档，优先修复路由和上下文交付，而不是复制第二份规则。
