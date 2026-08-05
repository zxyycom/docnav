---
title: 以完整适配器表面验证共享边界
status: active
alignment: unaligned
createdAt: 2026-08-05T12:41:14Z
purpose: 让 adapter 边界证据覆盖当前产品实际保留的选择、导航、读取与展示表面。
background: 旧方向仍把已经删除的 probe 列为完整证据，可能让未来工作为不再存在的 surface 保留空间。
decision: 完整证据覆盖当前选择链路、固定 operation、ref 与分页、结构化与全文读取、raw 与 readable 输出；格式需要专用展示时也要纳入验证。
relations:
  - type: 修订
    target: adapter-evolution/validate-boundaries-with-complete-adapter-behavior.md
---

## 目的
- 让共享 adapter 边界只建立在真实格式经过完整现行产品表面后暴露的共同职责上。
- 防止历史上已经删除的选择机制继续影响未来 adapter 设计。

## 背景
- 前序决策要求用完整 adapter 行为检验边界，这个方向仍然成立。
- 当前选择已经由 manifest pathname routing 承接，adapter contract 不再拥有 probe；继续把 probe 当作证据会与当前 owner 规范冲突。
- 固定 operation、ref 与分页、structured/full-read、raw/readable 输出会从不同责任层暴露共享职责和格式特例。

## 决策
- 采用: 一个真实 adapter 经过当前选择链路、`outline`、`read`、`find`、`info`、ref 与分页、structured/full-read、raw protocol 和 generic readable path 后，才形成完整边界证据；该格式需要专用 presentation 时也要完成相应验证。
- 采用: 行为可以按依赖关系由相连 change 依次交付；每个阶段记录实际暴露的多余职责、缺失职责、格式特例和共享摩擦。
- 不采用: 仅为保留历史检查清单而恢复 probe、兼容 surface 或提前增加共享抽象。
