---
title: 自定义渲染由可读视图拥有
status: active
alignment: unaligned
createdAt: 2026-07-28T11:58:11Z
purpose: 让格式专用的人类展示独立演进，并保持 raw protocol 与可继续导航事实稳定。
background: 信息密度、层级和 preview 是 presentation 选择，adapter operation result 是机器事实。
decision: 自定义渲染由 readable-view 拥有，并以既有 raw facts 作为唯一输入契约。
relations:
  - type: 修订
    target: navigation-output/separate-semantic-read-source-fidelity-and-custom-rendering.md
---

## 目的
- 允许 JSON 和后续格式为人类阅读调整信息密度、层级、标点、preview 与分页呈现。
- 保持 raw protocol、ref、cost、page 和 adapter operation result 可独立验证并兼容演进。

## 背景
- Raw output 面向稳定校验和调用方消费，readable output 面向终端阅读，两者复用业务事实但不复用传输包装。
- Structured serializer 决定 raw content spelling；custom renderer 决定这些 raw facts 的终端呈现。
- 每个格式的 presentation contract 可以独立于 raw protocol 审批和演进。

## 决策
- 采用: “自定义渲染”表示 `readable-view` 对既有 raw result facts 的面向人展示。
- 采用: 自定义渲染可以调整信息密度、层级、标点、preview 和分页显示；raw result、ref、cost、page 与协议传输包装继续作为机器契约。
- 采用: 每个格式用明确批准的 presentation contract 定义具体 readable output。
