---
title: 源码顺序采用格式级成本判断
status: active
alignment: aligned
createdAt: 2026-07-28T11:58:10Z
purpose: 让格式 owner 按可验证成本选择 source order 或确定性语义顺序。
background: 成员顺序对部分结构化文档有用，但不同 parser 和格式模型承载它的成本并不相同。
decision: 格式在私有表示可低成本自然承载时保留源码顺序，否则采用确定性语义顺序。
relations:
  - type: 修订
    target: structured-read-semantics/separate-semantic-read-source-fidelity-and-custom-rendering.md
---

## 目的
- 为用户可感知的源码顺序建立明确的格式级成本门槛。
- 让每个格式依据自身 parser 与私有表示选择稳定顺序。

## 背景
- Object member、mapping entry 或 declaration 的源码顺序可能提升阅读和导航体验，但通常不决定节点的格式语义身份。
- 某些 parser 或 adapter-private tree 会自然保序；另一些实现需要新的 parser、额外全量副本或广泛分支。
- 共享调用方需要确定性顺序；source order 是否成为格式 contract 由该格式的价值与实现成本共同决定。

## 决策
- 采用: 所有格式提供确定性顺序；source order 是格式级选择，而非共享 invariant。
- 采用: 格式 owner 用有界实验评估既有 parser、adapter-private 表示、memory、branching 和 maintenance 成本。
- 采用: 成本保持当前 model 量级时，把 source order 固化为该格式 contract；其它结果采用并文档化 parser/model 的确定性语义顺序。
- 采用: 顺序实验及其表示由格式 owner 承接，共享 contract 只要求确定性。
