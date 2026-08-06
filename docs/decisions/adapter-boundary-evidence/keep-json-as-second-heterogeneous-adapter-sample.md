---
title: 保留 JSON 作为第二个异构适配器样本
status: active
alignment: aligned
createdAt: 2026-08-06T03:39:43Z
purpose: 以范围可控且足够异构的格式继续承载第二个真实 adapter 边界样本，并让证据面随 Current 产品表面演进。
background: JSON 的样本价值仍成立，但历史记录把已删除的 probe 固定为证据面，不能继续准确描述当前完整 adapter surface。
decision: 第二个真实 adapter 继续选择 JSON；有效边界证据遵循当前完整 adapter surface，而不保留已删除的 probe，后续格式仍独立批准。
relations:
  - type: 修订
    target: adapter-boundary-evidence/select-json-as-second-adapter.md
---

## 目的
- 保留 JSON 作为第二个真实且结构不同的 adapter 样本，使 parser 复杂度不会主导共享边界结论。
- 让该样本证明当前实际保留的完整 adapter surface，而不是把已经删除的历史入口继续当成验收义务。

## 背景
- JSON 与 Markdown 都常见且可形成树；JSON 使用 object key、array index 和 value kind，Markdown 使用 heading 与 section。
- YAML 和 TOML 与 JSON 的结构较接近，却带来额外格式语义和 parser；代码、HTML、PDF 和表格虽能施加更强压力，也会把更大的解析与导航问题混入第二样本验证。
- JSON 的选择仍主要提供架构边界证据，不把尚未观察到的 Beta 用户需求补写成选择依据。
- 完整证据范围由活动决策 [以当前支持的完整适配器表面验证共享边界](validate-shared-boundaries-across-supported-adapter-surfaces.md)独立拥有，避免本记录再次复制会随产品演进的 surface inventory。

## 决策
- 采用: Docnav 的第二个真实 adapter 继续选择 JSON，因为它简单、常用、结构差异足够且实现范围可控。
- 采用: JSON adapter 必须经过当前完整 adapter surface、真实 CLI、core routing、registry、protocol 和 release 路径，才能构成有效边界证据；已经删除的 probe 不再形成验收义务。
- 采用: 架构边界证据与用户需求证据保持区分；该选择不虚构尚未观察到的 JSON 导航需求。
- 采用: 该选择只决定第二个样本；后续格式的优先级和内部模型由各自目标、场景与证据独立批准。
