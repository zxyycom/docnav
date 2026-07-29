---
title: 用完整 adapter 行为检验边界
status: active
alignment: unaligned
createdAt: 2026-07-29T01:32:38Z
purpose: 让 adapter 边界证据覆盖格式实现会实际经过的全部产品行为。
background: 解析、导航、查找、读取和可读展示会分别暴露共享设计中的多余职责、缺失职责与格式特例。
decision: 真实格式走通固定 operation、full-read 与格式专用 readable presentation 后，才形成完整 adapter 边界证据；各阶段可以由相连的 change 依次交付。
relations: []
---

## 目的
- 用完整格式行为检验现有 adapter、protocol、output 和交付边界。
- 让共享抽象的保留、删除与补充依据来自实际路径，而不是单一解析或导航切片。

## 背景
- Probe、outline、read、find、info、full-read 和 readable presentation 分别经过不同责任层。
- 完整边界证据需要合并 parser、ref、raw operation、presentation 与跨层映射各自暴露的问题。
- 自定义渲染需要独立确定信息密度、层级、preview、分页呈现和 owner mechanics，适合在 raw 行为稳定后继续验证。

## 决策
- 采用: 一个真实 adapter 依次走通 probe、outline、read、find、info、full-read 和格式专用 readable presentation 后，才形成完整的边界验证证据。
- 采用: 行为可以按依赖关系拆成多个相连 change；后续 presentation 阶段仍是边界验证的必需组成部分。
- 采用: 每个阶段记录实际暴露的多余职责、缺失职责、格式特例和共享摩擦，再据此决定共享设计的保留、删除或补充。
