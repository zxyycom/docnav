---
title: JSON 专用阅读展示属于近期完整交付
status: archived
alignment: null
createdAt: 2026-08-05T06:47:42Z
purpose: 让 JSON 在共享 raw 行为验证后获得真正适合阅读的格式专用展示。
background: Generic readable view 能证明共享输出链路可用，但不能长期替代 JSON 对路径、标点、preview 和分页信息密度的 presentation contract。
decision: JSON 专用 renderer 可以短暂等待底层 raw facts 稳定，但仍属于近期完整 adapter 交付，不得无限期停留在 generic presentation。
relations: []
---

## 目的
- 完成 JSON adapter 从 raw operation 可用到格式专用阅读体验可用的后续阶段。
- 在避免底层 contract 反复返工的同时，不把 generic renderer 误当作永久产品展示。

## 背景
- Generic `readable-view` 已能消费 JSON 的共享 protocol facts，因此适合作为 adapter 接入和 raw 路径验收阶段。
- JSON 的路径定位、标点、escaping、preview、分页和信息密度仍需要独立批准的 presentation contract。
- 既有长期决策已明确自定义渲染由 readable-view 拥有，且完整 adapter 边界证据包含格式专用 presentation。

## 决策
- 采用: `add-json-readable-renderer` 在相关 raw facts 足够稳定后进入近期实施，不要求抢在基础契约之前完成，也不得无限期延后。
- 采用: Generic renderer 只作为共享输出验收和迁移阶段；JSON 的最终 readable presentation 由独立 output-owned contract 承接。
- 不采用: 因 generic view 当前可用就把 JSON 专用 presentation 从完整 adapter 交付中永久删除。
