---
title: JSON 引用采用规范化树路径
status: archived
alignment: null
createdAt: 2026-07-28T11:57:52Z
purpose: 让每个可导航 JSON 节点拥有唯一、可解释且能安全经过 CLI argv 的 adapter-owned ref。
background: JSON 天然以树路径定位节点，但 raw pointer 可包含控制字符，生成式 ID 又会丢失路径语义。
decision: 使用带 json 前缀的 RFC 6901 URI fragment 表示，并由 JSON adapter 独占生成、解析和校验。
relations: []
---

## 目的
- 让 `outline -> ref -> read` 对 JSON object member、array element、root 和特殊 key 都能稳定 roundtrip。
- 让 JSON adapter 拥有 ref grammar 与错误，shared/core 执行 opaque pass-through。

## 背景
- Decoded object key 唯一时，object key 与 array index 组成的 JSON tree path 可以唯一定位节点。
- RFC 6901 raw pointer 允许 token 含 NUL 等控制字符，这些字符不能可靠经过 CLI argv；URI fragment representation 已提供 UTF-8 percent encoding。
- Adapter prefix 加 RFC 6901 URI fragment 同时提供格式所有权、标准 escape 和 CLI-safe 表示。

## 决策
- 采用: JSON root ref 为 `json:#`，其它节点使用 `json:#<RFC 6901 URI fragment>`；object token 先做 JSON Pointer escape，再做 canonical URI fragment percent encoding。
- 采用: 生成的 ref 必须非空、ASCII-safe 且 canonical，并能区分 root、空 object key、object 数字 key 和 array index。
- 采用: JSON adapter 负责 ref grammar、canonical validation、解析和 resolution；shared/core 只原样传递。
- 采用: Grammar 非法映射为 invalid ref，grammar 合法但当前树中不存在节点映射为 not found。
