---
title: JSON 查找使用源码文本语料
status: active
alignment: aligned
createdAt: 2026-07-29T01:32:38Z
purpose: 让 JSON find 命中用户在文件中实际写下的文本。
background: Canonical ref 和 structured read 会转换 pointer escape、空白、字符串 escape 与普通 scalar spelling，不能代表原文搜索语义。
decision: JSON find 对去除可选 UTF-8 BOM 后的原文执行 literal search，并把源码命中映射为 JSON adapter 拥有的可继续读取 ref。
relations: []
---

## 目的
- 让查询与 JSON 文件中的实际 spelling、空白、escape 和结构文本保持一致。
- 让每个 find 结果继续提供可原样传给 read 的 adapter-owned ref。

## 背景
- Canonical JSON Pointer 是节点身份表示，会转义 object member token。
- Structured read 是选中 value 的规范化序列化，会改变空白、部分 escape 和普通 scalar spelling。
- 原文是用户执行文本查找时唯一直接可见且无需推导的语料。

## 决策
- 采用: JSON find 以去除一个可选开头 UTF-8 BOM 后的 UTF-8 原文作为搜索语料。
- 采用: 非空 query 按 source-text literal semantics 匹配，命中顺序和位置来自原文。
- 采用: JSON adapter 将每个源码命中确定性映射到可继续读取的 JSON ref；精确归属、label、location、分页和重复命中规则由 JSON 行为 owner 完整定义。
