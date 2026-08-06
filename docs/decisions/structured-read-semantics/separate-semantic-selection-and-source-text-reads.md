---
title: 分离语义 selection 与完整 source text 读取
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:39Z
purpose: 让 structured read 物化格式语义，同时为需要完整 source text 的调用方保留独立入口。
background: Adapter-defined selection 不限于 logical node，语义物化与源码文本保真具有不同责任。
decision: Structured read 返回 selection 的格式语义表示，unstructured full-read 返回格式 owner 定义的完整 source text。
relations:
  - type: 修订
    target: structured-read-semantics/separate-structured-and-source-reads.md
---

## 目的
- 让 structured read 稳定物化 producer 所记录的 adapter-defined selection。
- 为需要完整源码 spelling 和布局的调用方保留权责清楚的 source-text 路径。

## 背景
- Ref 可以表达节点、区域、位置、查询或其它 adapter 私有概念，shared contract 不应把 selection 收窄为 node。
- Structured semantics 与完整 source text 面向不同调用方，并允许格式 owner 定义各自必要的规范化或保真边界。
- Byte-exact source 是比当前 text result 更强的独立能力，不能从“完整原文”措辞隐式推导。

## 决策
- 采用: Structured read 物化 adapter-defined selection 的格式语义表示；格式 owner 定义会影响 value、identity、correspondence 或可用性的例外。
- 采用: Unstructured full-read 返回格式 owner 定义的完整 source text，并拥有该文本的 spelling 与整体布局语义。
- 采用: Structured read 不承担普通 whitespace、escape spelling 或任意 source slice 保真；需要这些 source-text 事实的调用方使用 full-read。
- 采用: 需要 byte-exact source、原始编码或当前 full-read 未声明事实的调用方，必须使用另行批准的能力，不能把这些保证附加到 structured read。
