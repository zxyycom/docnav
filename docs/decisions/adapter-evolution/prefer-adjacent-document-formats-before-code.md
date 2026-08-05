---
title: 复杂代码适配器前先扩展相邻文档格式
status: active
alignment: unaligned
createdAt: 2026-08-05T06:47:42Z
purpose: 用复杂度递增的真实文档格式继续验证 adapter 边界并控制扩展风险。
background: YAML、TOML 等格式更接近现有文档导航模型，代码导航则同时引入语言 parser、符号语义、依赖和更强替代工具竞争。
decision: 下一阶段格式扩展优先考虑简单文档类 adapter，代码 adapter 长期延后；每个具体格式仍需按自身场景和证据独立批准。
relations: []
---

## 目的
- 在扩展格式覆盖面的同时，让每个新 adapter 主要检验 Docnav 边界，而不是让 parser 和领域语义复杂度主导工作。
- 保持 Docnav 近期聚焦文档导航，再逐步进入代码等复杂信息源。

## 背景
- JSON 已提供树状结构化文档样本，但后续 adapter 仍需验证格式识别、ref、分页、原文保真和 readable presentation 的可扩展性。
- YAML、TOML 等简单文档格式可在相邻模型上增加真实格式语义压力。
- 代码 adapter 需要语言 parser、符号模型、源码区域与依赖治理，同时代码生态已有成熟结构化探索工具。

## 决策
- 采用: 在复杂代码 adapter 之前，下一阶段优先评估 YAML、TOML 或同类简单文档格式。
- 采用: `add-ast-grep-code-adapter` 保持长期方向，不进入近期格式扩展排序，也不反向驱动当前共享 adapter contract。
- 采用: 具体下一个格式仍由独立目标、用户场景、parser 风险和完整行为证据批准；本决策只规定复杂度方向，不预先指定唯一格式。
- 不采用: 仅因代码 adapter artifacts 已存在或 routing 前置已完成，就跳过相邻文档格式直接进入复杂符号导航。
