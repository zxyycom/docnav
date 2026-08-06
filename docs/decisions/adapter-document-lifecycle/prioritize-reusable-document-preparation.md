---
title: 优先建立调用内可复用文档准备边界
status: archived
alignment: null
createdAt: 2026-08-05T06:47:43Z
purpose: 让 adapter 最小生命周期支持通用组合并避免同一文档在一次调用中反复完整解析。
background: Selection、operation 和 core composition 会重复 acquisition、decode、parse 或索引，既增加成本也妨碍后续通用能力与新 adapter 接入。
decision: 近期优先建立 invocation-private 的可复用准备边界和最小 adapter API，同时保留格式语义私有并拒绝公共状态标识、跨调用缓存和通用状态注册表。
relations: []
---

## 目的
- 消除同一 invocation、同一兼容 document view 上因阶段分离造成的重复完整准备。
- 收敛 adapter 必须实现的最小生命周期和 operation 边界，为 outline、find、read 等通用组合提供稳定基础。
- 让后续简单文档 adapter 复用经过 Markdown 与 JSON 验证的边界，而不复制 core orchestration。

## 背景
- 当前多个阶段会独立 acquisition、decode、parse 或构造索引；composition 增加 operation 次数时会放大重复工作。
- 这不仅是局部性能优化，还会影响 adapter 最小 API、navigation 生命周期、source snapshot、cleanup 和通用功能责任。
- 具体 Rust 形状、snapshot 和 cleanup 规则仍需在关联 change 中用 Markdown、JSON 和真实组合路径完成决策与验证。

## 决策
- 采用: `reuse-adapter-document-state` 作为近期基础方向推进，目标是让 selected adapter 在一次 invocation 内复用同一兼容 document view 的 acquisition、decode、parse、index 和 source-region facts。
- 采用: Navigation 控制 invocation 生命周期和通用组合；adapter 继续拥有格式检测、解析、ref、私有表示和 operation 语义。
- 采用: 共享 adapter API 只承接 Markdown、JSON 和真实组合路径共同证明的最小职责；精确机制由关联 change 的架构门禁批准。
- 不采用: 把复用状态放入 protocol、ref、continuation、日志或 caller-visible identifier，也不引入跨 invocation cache、通用 state registry 或为未来外部 host 预造的抽象。
