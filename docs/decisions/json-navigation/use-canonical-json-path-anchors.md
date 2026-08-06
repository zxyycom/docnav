---
title: JSON 引用使用规范化路径锚点
status: active
alignment: aligned
createdAt: 2026-08-06T02:50:39Z
purpose: 让 JSON ref 保持精确、可往返的路径 grammar，同时允许多种 selection view 共享同一 logical anchor。
background: Current JSON 已有 base、direct-comment 和 tail-comment 三种 view，旧有每节点唯一 ref 假设不再成立。
decision: JSON ref 由精确 view prefix 与 canonical RFC 6901 URI-fragment path anchor 组成，多种 view 可以共享 anchor。
relations:
  - type: 修订
    target: json-navigation/use-canonical-json-pointer-refs.md
---

## 目的
- 让 JSON object member、array element、root 和特殊 key 使用稳定、CLI-safe 且可由 read 往返的 canonical logical path。
- 保留 base、direct-comment 与 tail-comment selection 的明确 grammar，同时纠正 ref 与 logical value 一对一的旧假设。

## 背景
- Decoded object key 和 array index 可以组成唯一 logical path；RFC 6901 URI-fragment 表示提供标准 pointer escape 与安全传输。
- 同一 logical path 可以锚定 base value、direct-comment bundle 或 tail-comment bundle，多个 ref 可以共享 anchor，但仍分别拥有 canonical grammar 和 producer/read correspondence。
- Shared/core 只需要 opaque pass-through，不应从 prefix、path 或 presentation facts 重建 selection。

## 决策
- 采用: Base、direct-comment 和 tail-comment refs 分别使用 `json:#<fragment>`、`json:comments:#<fragment>` 和 `json:tail-comments:#<fragment>`；root refs 分别以三个 prefix 后的 `#` 结束。
- 采用: 三种 view 复用同一个 RFC 6901 URI-fragment path anchor；object token 先做 JSON Pointer `~0` / `~1` escape，再对 UTF-8 bytes 做 canonical 大写 hexadecimal percent encoding。
- 采用: 生成 ref 必须非空、ASCII-safe、不含 raw NUL/control character，并能区分 root、空 object key、object 数字 key和 array index；array index grammar 继续保持 canonical。
- 采用: JSON adapter 独占 view/pointer grammar、canonical validation、生成、解析、resolution 与 producer/read correspondence；compatible view 上 producer 发出的完整 ref 必须可由 read 使用。
- 采用: Prefix、escape 或 index grammar 非法映射为 `REF_INVALID`；anchor canonical 但 path 或所选 bundle 不存在映射为 `REF_NOT_FOUND`。
- 采用: Shared protocol、core 和其它调用入口只校验并原样传递非空 ref，不解析 anchor、view 或 selection identity。
