---
title: 选择 JSON 作为第二个真实适配器
status: active
alignment: aligned
createdAt: 2026-07-28T11:57:52Z
purpose: 以范围可控且足够异构的格式建立第二个真实 adapter 边界样本。
background: JSON 简单常用，树导航不同于 Markdown，又比代码、HTML 或 PDF 更少混入解析复杂度。
decision: 第二个真实 adapter 选择 JSON，后续格式继续按自身目标和证据独立选择。
relations: []
---

## 目的
- 选择一个能真实施压 adapter contract、且 parser 复杂度不会主导结论的第二格式。
- 让 ref、outline、read、find、info、probe、registry 和发布链路都经过非 Markdown 样本验证。

## 背景
- JSON 与 Markdown 都常见且可形成树；JSON 使用 object key、array index 和 value kind，Markdown 使用 heading 与 section。
- YAML 和 TOML 与 JSON 的结构较接近，却带来额外格式语义和 parser；代码、HTML、PDF 和表格虽能施加更强压力，也会把更大的解析与导航问题混入首次边界验证。
- 该选择的证据类型是架构边界验证；Beta 使用中尚未观察到 JSON 导航需求。

## 决策
- 采用: Docnav 的第二个真实 adapter 选择 JSON，因为它简单、常用、结构差异足够且实现范围可控。
- 采用: JSON adapter 必须走与 Markdown 相同的真实 CLI、core routing、registry、protocol 和 release 路径，才能构成有效边界证据。
- 采用: 该选择只决定第二个样本；后续格式的优先级和内部模型由各自目标、场景与证据决定。
