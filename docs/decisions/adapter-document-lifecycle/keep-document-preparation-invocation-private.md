---
title: 调用内文档准备保持适配器私有
status: active
alignment: aligned
createdAt: 2026-08-06T02:35:41Z
purpose: 让同一调用复用兼容文档视图，同时避免建立公共或跨调用状态。
background: 文档准备复用会影响 adapter 生命周期和职责边界，不只是局部性能实现。
decision: Adapter 拥有调用内私有文档视图，navigation 拥有其生命周期和通用组合，状态不进入公共或跨调用契约。
relations:
  - type: 修订
    target: adapter-document-lifecycle/prioritize-reusable-document-preparation.md
---

## 目的
- 让同一 selected invocation 中需要文档事实的阶段复用一个兼容视图，避免重复完整准备。
- 保持格式语义、文档状态和 ref 定位事实私有，不增加 caller-visible session 或跨调用状态面。

## 背景
- Acquisition、decode、parse、索引和 source-region 准备会共同影响 adapter 最小生命周期、ref 一致性和 operation 组合。
- Markdown 与 JSON 已证明调用内 document owner 能承接这些职责；具体表示和准备步骤仍会按格式演进。
- 当前机制、Rust 类型、cleanup 和索引形状属于 owner 规范、实现与 change 证据，不需要固化为长期决策。

## 决策
- 采用: Final selection 和输入校验完成后，navigation 为 selected adapter 创建调用内私有 document owner，并控制其使用范围、通用组合和释放时机。
- 采用: Adapter 在该边界内拥有文档 acquisition、decode、parse、私有表示、格式语义、ref 和 operation 行为；同一调用的 eligible stages 复用兼容视图。
- 采用: 私有文档状态不得进入 protocol、ref、continuation、日志、caller-visible identifier、全局状态注册表或跨调用 cache。
- 边界: 精确 factory、lazy preparation、cleanup、索引和共享 API 形状由当前 owner 规范及真实实现证据决定。
