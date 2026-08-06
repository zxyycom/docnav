---
title: JSON value 使用确定性语义序列化
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:38Z
purpose: 让所有 JSON selection view 以同一确定性 spelling 表达 logical value，同时明确普通 source lexeme 不保真。
background: Current JSON value 使用 pinned serializer 与 two-space layout，number、member identity 和 order 另有独立约束。
decision: JSON logical value 使用 pinned serializer 的 two-space 表示，所有 selection view 复用并保留既定正确性例外。
relations:
  - type: 修订
    target: json-navigation/normalize-structured-json-output.md
---

## 目的
- 为 selected logical JSON value 提供确定性、可分页并适合协议校验的 strict-JSON 文本表示。
- 保留已批准的 layout、serializer spelling 和 dependency-review 约束，同时明确普通 source lexeme 与格式正确性事实的不同责任。

## 背景
- 普通 whitespace、string escape、boolean/null spelling 和 trailing newline 可以在不改变 JSON value 的情况下由 pinned serializer 规范化。
- Number token、member identity、source order 和其它会改变 value、path 或可用性的事实具有独立格式约束。
- Base、direct-comment 与 tail-comment view 都会物化同一 logical value；comment projection 不应建立第二套 value serializer contract。
- 可观察 spelling 的 owner 是 JSON contract；pinned parser/serializer 是当前实现机制和升级复核点，不是独立契约 owner。

## 决策
- 采用: JSON logical value 使用 workspace-pinned parser/serializer 的自然 string/scalar spelling，并对 container 使用 two-space pretty layout 和既有 terminal newline 语义。
- 采用: 普通 whitespace、string escape、boolean/null 与 trailing-newline source spelling 不保真；需要这些 source-text 事实的调用方使用 full-read。
- 采用: Base、direct-comment 与 tail-comment view 复用同一 strict-JSON value serialization；view-specific comment material 保持自己的 JSON projection 语义。
- 采用: Raw number token、decoded member identity、source order 和其它正确性例外继续由相应 JSON 决策与 Current owner 约束。
- 采用: 影响可观察 value spelling 或 layout 的 parser/serializer 升级必须复核 JSON owner contract 和回归证据。
