---
title: JSON 深度上限保持适配器私有
status: active
alignment: aligned
createdAt: 2026-07-28T11:57:53Z
purpose: 用 adapter-private 单一安全边界控制 JSON 递归成本并保持公共输入契约稳定。
background: JSON 需要确定性深度保护，当前运行证据支持固定边界而非 caller-configurable 参数。
decision: 根深度定义为零，当前最大深度固定为 127，并由 adapter-private 单一硬编码配置源拥有。
relations: []
---

## 目的
- 为 parse、traversal 和递归处理建立明确、可测试的资源安全边界。
- 让安全常量由一个 owner 派生到 probe、operation 和测试，同时保持公共 input inventory 稳定。

## 背景
- 深度上限是格式实现的安全策略，不等同于用户导航选项。
- 单一 adapter-private 配置可以让 parser、operation 和测试共享同一事实来源。
- 当前使用证据支持一个常见、可明确测试的固定值；动态配置需要新的调用方场景和兼容性依据。

## 决策
- 采用: JSON root depth 定义为 `0`，当前最大支持 depth 固定为 `127`。
- 采用: 该值由 JSON adapter-private 单一硬编码配置源拥有，probe、operation 和测试从同一 owner 派生。
- 采用: Caller-visible CLI、env、config、protocol 和 shared parameter inventory 保持当前契约。
- 采用: 新的调用方需求或资源证据通过后续决策修订数值或配置面。
