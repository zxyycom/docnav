---
title: 展示契约批准后推进 JSON 专用阅读输出
status: active
alignment: unaligned
createdAt: 2026-08-06T02:50:39Z
purpose: 把 JSON 专用 readable presentation 保持为独立后续里程碑，并以可观察展示契约批准作为实施门禁。
background: Generic readable view 已证明共享输出链路，但不会替代 JSON 专用展示；当前阻塞项是 presentation contract，而非相邻 raw work。
decision: JSON 专用 renderer 在展示契约门禁关闭后进入实施，相邻 raw change 不是前置，完成前继续保持 Planned。
relations:
  - type: 修订
    target: product-direction/complete-json-readable-presentation.md
---

## 目的
- 完成 JSON 从 generic shared presentation 到经批准格式专用阅读体验的独立后续里程碑。
- 用可观察 contract 门禁替代“raw facts 足够稳定”等不可恢复的模糊启动条件。

## 背景
- Generic `readable-view` 已验收同一 `ProtocolResponse` 的共享阅读路径，但没有决定 JSON 专用信息密度、标点、preview、分页或 renderer selection。
- `add-json-readable-renderer` 已把这些 presentation 与 selection 问题列为实施前必须关闭的显式门禁。
- Token cost、find result、document state 或 performance workstream 不构成本 change 的统一前置；其中先落地的 Current 变化只触发 scoped re-audit。

## 决策
- 采用: JSON 专用 renderer 继续作为展示契约门禁关闭后应推进的独立、output-owned 产品里程碑；generic renderer 不成为永久替代品，也不重新打开已完成的 raw JSON adapter 验收。
- 采用: 开始实现前必须明确批准适用 operation/branch、稳定字段和信息密度、标点与 escaping、完整 opaque ref 的定位信号、preview、page/continuation 以及 renderer selection mechanics，并同步为可证伪 contract 与任务。
- 采用: Presentation 只能消费同一个 immutable `ProtocolResponse`，保持 protocol、JSON raw result、ref、ordering、cost、page、schema/example 和 public output values 不变。
- 采用: 相邻 raw 或 performance changes 不是实施前置；门禁关闭时按届时 Current 输入做范围内重核即可。
- 采用: 在 owner、实现、contract tests、真实 CLI 和 package 证据完成前，JSON 专用 renderer 保持 Planned，本决策保持 unaligned。
