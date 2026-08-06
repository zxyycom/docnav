---
title: 规范化 JSON 结构化读取输出
status: archived
alignment: null
createdAt: 2026-07-28T11:58:10Z
purpose: 让 raw structured JSON 以稳定 serializer spelling 和布局表达选中 value。
background: JSON 解析后空白和 escape spelling 会自然规范化，而数字、重复 key 与顺序另有独立正确性边界。
decision: 使用 pinned parser 和 serializer 的自然结果及两空格布局，只保留由独立决策要求的格式事实。
relations:
  - type: 修订
    target: structured-read-semantics/separate-semantic-read-source-fidelity-and-custom-rendering.md
---

## 目的
- 为 JSON selected value 提供确定性、可分页且适合协议校验的结构化文本。
- 让 workspace-pinned parser/serializer 成为普通 lexeme spelling 的唯一 owner。

## 背景
- `"\u0061"` 与 `"a"`、不同 whitespace 和尾随换行可以表达相同 JSON value，解析再序列化时通常会采用 serializer 的标准 spelling。
- Number token、重复 key 和 object order 会影响值、identity 或可用性，分别由独立决策处理。
- Structured output 的可观察 spelling 受 workspace-pinned dependency 影响，因此依赖升级属于可观察行为复核点。

## 决策
- 采用: JSON structured read 使用 workspace-pinned parser/serializer 的自然 scalar 与 string escape 表示，并对 container 使用两空格 pretty layout。
- 采用: 普通 whitespace、string escape、boolean/null 和尾随换行 spelling 以 serializer 输出为准。
- 采用: 原始 number token、重复 member handling 和 object source-order 策略继续由各自独立决策约束。
- 采用: 影响可观察 structured output 的 parser/serializer 升级必须复核 owner contract 和回归证据。
