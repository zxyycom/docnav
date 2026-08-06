---
title: 格式专用展示保持在可读输出层
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:39Z
purpose: 让格式专用 presentation 独立演进，同时保持 raw protocol 和导航事实稳定。
background: 机器结果与面向人的信息密度、标点和 preview 属于不同消费层。
decision: 格式专用 presentation 只能在 Rendered 路径消费同一个 immutable ProtocolResponse，不改变 raw machine contract。
relations:
  - type: 修订
    target: readable-presentation/keep-custom-rendering-in-readable-view.md
---

## 目的
- 允许不同格式按阅读需求调整可读输出，而不让展示偏好反向改变机器契约。
- 让 raw 与 readable 两条输出路径可以独立验证和演进。

## 背景
- `ProtocolResponse`、ref、cost、page 和 operation result 是稳定机器事实。
- 信息密度、标点、preview、framing 和其它面向人的组织属于 presentation contract。
- 每个格式的具体 presentation 仍需要由对应 output owner 或 change 单独批准。

## 决策
- 采用: 格式专用 presentation 只存在于 `Rendered` / `readable-view` 路径，并把已经形成的同一个 immutable `ProtocolResponse` 作为完整输入 contract；不建立可替代 response 的第二套 facts contract。
- 采用: Raw 与 readable 复用业务事实但不复用传输包装；structured serializer 决定 machine content spelling，renderer 只决定面向人的 presentation。
- 采用: Renderer 可以改变面向人的组织和文本，但不得改变 raw protocol result、ref、cost、page、错误身份或 protocol transport wrapping。
- 采用: 每个格式的具体 readable presentation、selection mechanics 和验收证据由独立 output-owned contract 承接。
