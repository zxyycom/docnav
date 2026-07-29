---
title: 用异构真实适配器检验共享抽象
status: active
alignment: unaligned
createdAt: 2026-07-28T11:57:52Z
purpose: 让共享 adapter contract 只承接由异构真实实现共同证明的职责。
background: 只有 Markdown 一个真实实现时，共享职责与 Markdown 特有实现之间没有可反驳的边界样本。
decision: 用第二个真实且结构不同的 adapter 检验边界，并只从重复出现的职责提炼共享抽象。
relations: []
---

## 目的
- 让 adapter abstraction 建立在真实格式差异和共同消费者义务上。
- 为共享 contract、framework 或扩展点建立可复核的多实现证据门槛。

## 背景
- 只有 Markdown 一个真实 adapter 时，任何 Markdown-specific 选择都可能被误认为通用 adapter 义务。
- 第二个真实格式会对 ref、tree traversal、read、find、probe 和 registry 施加不同约束，形成完整产品路径的边界证据。
- 架构验证证据与用户格式需求证据回答不同问题，需要分别标注。

## 决策
- 采用: 在继续推广 adapter 共享抽象前，引入至少一个结构和导航语义不同的真实格式 adapter，以完整产品路径检验现有边界。
- 采用: 第二个 adapter 首先复制既有最小 contract 形状并暴露摩擦；只有跨真实实现重复出现、且消费者共同依赖的职责才进入 shared abstraction。
- 采用: 用户格式需求证据与架构边界验证证据分别记录；任一依据都按其实际证明范围支撑 change。
