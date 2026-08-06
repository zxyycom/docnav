---
title: 探索材料不形成交付承诺
status: active
alignment: aligned
createdAt: 2026-08-05T06:47:42Z
purpose: 让方向探索与正式 change、实施优先级和产品承诺保持清楚边界。
background: Operation composition 材料用于持续挖掘候选体验，但宽泛候选和开放问题不能直接充当可实施契约。
decision: 探索 change 只保存想法和筛选输入，不形成实施授权、依赖或交付排序；具体能力成熟后必须由独立且精确的 change 承接。
relations: []
---

## 目的
- 允许持续记录未来组合方向，而不让想法库存被误读为活动产品工作。
- 要求进入实现的能力拥有独立目标、owner、public contract 和验收边界。

## 背景
- `explore-operation-composition` 同时包含多种 convenience 候选，且刻意不选择最终 surface、schema 或 owner。
- 把探索任务数量、artifact 完整度或 change 状态当作交付信号，会让模糊方向提前约束正式实现。

## 决策
- 采用: `explore-operation-composition` 是持续挖掘想法的探索载体，不是正式 implementation change。
- 采用: 探索材料不形成其它 change 的实现依赖、优先级或验收义务；成熟候选必须由新的或明确重写后的精确 change 承接。
- 不采用: 为了清空探索任务而把多个未收敛候选合并实现，或直接复用探索文本作为稳定 public contract。
