---
title: JSON 数字保留已验证源码 token
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:38Z
purpose: 让合法 JSON number 在解析和 value 序列化之间保持同一文本身份。
background: 常见整数或浮点模型不能无损承载全部合法 JSON number token。
decision: JSON adapter 保存语法已验证的原始 number token，并在 logical value 序列化时原样写回。
relations:
  - type: 修订
    target: json-navigation/preserve-number-tokens.md
---

## 目的
- 避免整数溢出、浮点舍入或 canonical number 改写改变用户文档中的合法 number identity。
- 把数值计算与结构化文档导航的文本身份分离。

## 背景
- 合法 JSON number 可以超出常见整数范围，也可以使用小数或指数 spelling。
- Navigation 需要可靠物化已选择的 JSON value，但不需要定义 arithmetic equivalence。
- Find label 和 source excerpt 有自己的语料、预算与截断语义，不承担完整 number identity 证明。

## 决策
- 采用: JSON decode model 保存每个 number 的原始、语法已验证 token，并在 logical value serialization 中原样写回。
- 采用: Arithmetic equivalence、数值计算和 normalization 位于 JSON navigation 能力之外。
- 边界: Token 捕获和写回机制保持 adapter-private；find 的 source spelling 与 label 由 JSON find owner 独立定义。
