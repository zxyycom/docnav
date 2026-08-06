---
title: JSON 对象成员名必须唯一
status: active
alignment: aligned
createdAt: 2026-07-28T11:57:52Z
purpose: 让成功接收的 JSON object 具有唯一 decoded member name 和唯一 tree path。
background: JSON 语法可包含重复 name，而 parser 对重复 member 的处理并不一致。
decision: JSON adapter 只接受 decoded member name 唯一的 object，使成功 parse 保留全部成员和唯一路径。
relations: []
---

## 目的
- 让一个 canonical JSON path 对成功接受的文档最多指向一个 logical node。
- 让成功 parse 表示所有输入 member 都已保留且可由 tree path 唯一访问。

## 背景
- RFC JSON interoperability guidance 建议 object name 唯一，但语法上仍可能出现重复 name。
- 常见 producer 从 map、dict 或 struct 生成时通常会产生唯一 key；手写、拼接输入和 parser policy 仍可能形成重复 member。
- 若同一 object 中存在 decoded name 相同的多个 member，JSON Pointer 无法唯一表达它们的身份。

## 决策
- 采用: JSON adapter 必须在每一层 object 中按 decoded member name 检测重复；成功输入的 member name 唯一，重复输入被拒绝。
- 采用: Unicode escape 等不同源码 spelling 解码为相同 name 时仍视为重复。
- 采用: 检测属于 JSON adapter 的 decode 责任，成功结果采用唯一 member 与唯一 path 语义。
