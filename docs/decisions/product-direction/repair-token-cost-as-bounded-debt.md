---
title: 将 token cost 作为有界性能债务修复
status: active
alignment: unaligned
createdAt: 2026-08-05T06:47:42Z
purpose: 修复已知 token 计算性能问题，同时让治理成本与实际极端影响相称。
background: 既有性能测试已发现当前 calculator 开销明显，主要风险集中在大型选择和极端输入，而普通路径通常影响较小。
decision: Token cost 作为已知性能债务单独修复，目标是只对返回事实做有界估算并覆盖极端证据，但不自动成为所有产品 change 的统一前置。
relations: []
---

## 目的
- 防止 token cost 计算隐藏地扫描、序列化或 tokenize 未返回内容，从而破坏有限导航流程。
- 用与已知影响范围相称的证据和实现处理性能债务，避免把局部极端问题扩大成全局阻塞。

## 背景
- 过往性能测试已经确认当前 token calculator 存在性能问题，不再把是否有问题视为纯推测。
- 明显影响主要出现在大型 selection、结构化文档或其它极端输入；普通读取路径通常不是同等严重。
- Exact tokenizer parity 对 Docnav 的导航选择价值有限，却可能增加启动、计算、内存和依赖成本。

## 决策
- 采用: `redesign-token-cost-estimation` 作为已知性能债务推进，而不是等待通用性能审计重新证明问题存在。
- 采用: 修复目标是只对实际返回内容或当前可见 selection 形成有界、明确标注的估算；不得仅为 cost 读取、完整物化或 tokenize 未返回内容。
- 采用: 验证重点覆盖已知极端形状及普通路径回归，并记录修复前后资源证据。
- 不采用: 因极端输入存在问题就让 token estimator 自动成为所有独立产品 change 的统一前置。
