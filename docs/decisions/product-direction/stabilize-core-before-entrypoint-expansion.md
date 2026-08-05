---
title: 核心契约稳定后再扩张接入与交互面
status: active
alignment: unaligned
createdAt: 2026-08-05T06:47:42Z
purpose: 在基础文档契约收敛前避免接入层和人类交互面放大变更成本。
background: MCP 与交互式 CLI 会映射仍在演进的 adapter、find、protocol 和 output surface，过早接入会扩大后续调整范围。
decision: MCP bridge 与 interactive outline 保持长期方向，待基础契约稳定后重新审计和排序，不因 change 已存在或门禁可关闭而提前实施。
relations: []
---

## 目的
- 让新的调用入口建立在已经稳定的文档业务契约上，而不是把当前变化复制到更多 surface。
- 保留 MCP 和人类交互能力的长期方向，同时控制近期返工半径。

## 背景
- MCP bridge 需要映射 CLI 参数、protocol result、find shape、output schema 和错误语义。
- Interactive outline 会增加 human-only orchestration、依赖和终端行为，但不改善当前 agent 主路径的基础契约。
- Change 已经存在、规划材料完整或实施门禁可以关闭，只能说明候选可继续评估，不能证明产品时机已经成熟。

## 决策
- 采用: `implement-docnav-mcp-bridge` 长期延后；只有 adapter lifecycle、find、protocol 和 output 等基础契约稳定后，才从届时 Current 基线重新审计其 tool schema、映射和分发边界。
- 采用: `interactive-outline-selection` 只保留为未来方向，不进入近期实施排序，也不作为当前基础架构的设计输入。
- 不采用: 以现有 OpenSpec change 的存在、完成度或门禁解除作为扩大接入面和交互面的充分理由。
