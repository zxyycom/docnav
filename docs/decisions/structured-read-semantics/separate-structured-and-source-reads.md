---
title: 分离结构化语义读取与原文读取
status: archived
alignment: null
createdAt: 2026-07-28T11:57:53Z
purpose: 让结构化导航返回节点语义，同时为需要完整源码 spelling 和布局的调用方保留明确入口。
background: 语义序列化与逐字节源码保真面向不同读取需求，并需要不同的 adapter 数据责任。
decision: Structured read 拥有选中节点的格式语义，unstructured full-read 拥有完整原文。
relations:
  - type: 修订
    target: structured-read-semantics/separate-semantic-read-source-fidelity-and-custom-rendering.md
---

## 目的
- 让 structured read 稳定返回选中节点的格式语义。
- 为确实需要 whitespace、escape spelling、顺序或整体布局的调用方保留权责清楚的原文读取路径。

## 背景
- JSON、代码和后续结构化格式通常先解析源码，再以格式语义执行 navigation；重新序列化时部分词法细节会自然规范化。
- Structured read 的消费者需要节点内容和可继续 ref；full-read 的消费者需要完整 source spelling 与布局。
- 两种读取职责独立演进，格式 owner 只为会改变值、identity 或可用性的事实定义 structured-read 例外。

## 决策
- 采用: Structured read 返回选中节点的格式语义表示；格式 owner 明确声明会改变值、identity 或可用性的保真例外。
- 采用: Unstructured full-read 返回完整原文，并拥有源码 spelling 与整体布局的保真职责。
- 采用: 需要原始文本的调用方使用 full-read；structured read 的共享契约保持语义级。
