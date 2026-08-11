---
title: 探索材料不形成交付承诺
status: active
alignment: aligned
createdAt: 2026-08-05T06:47:42Z
purpose: 让方向探索、长期决定与正式 Change 的职责和进入条件保持清楚边界。
background: 历史 `explore-operation-composition` 材料同时保存多种候选体验和开放问题，能够支持方向判断，但没有收敛成一个可实施目标。
decision: 历史 operation-composition 只作归档输入；成熟的跨 change 判断进入决策，具体交付仅在用户明确要求后由精确 Change Plan 承接。
relations: []
---

## 目的
- 允许继续探索未来组合方向，而不让想法库存被误读为活动计划或产品承诺。
- 把已经成熟的长期判断和一次具体交付分给不同 owner。

## 背景
- 历史 `explore-operation-composition` 同时包含多种 convenience 候选，且刻意不选择最终 surface、schema 或 owner。
- 这类材料可以支持“探索不等于交付”等长期判断，但不能把宽泛候选整体迁移成活动 Change。
- OpenSpec 退役后，原材料已经随完整历史进入 `archive/legacy/openspec/`，不需要在 `changes/` 中维护第二份入口。

## 决策
- 采用: `explore-operation-composition` 只作为历史探索输入保留在 legacy archive；当前不建立同名 active Change。
- 采用: 已经成熟且跨 change 持续有效的判断由本决策及后续 decision records 承接；尚未成熟的想法留在当次讨论，用户明确要求沉淀调查时才进入 investigation report。
- 采用: 某个候选收敛出独立目标、owner、contract 和验收边界后，可以建议建立精确 Change；只有用户明确要求创建或维护时才写入持久 Change Plan。
- 不采用: 为了清空探索材料而把多个未收敛候选合并实现、维持宽泛活动 Change，或直接复用探索文本作为稳定 public contract。
