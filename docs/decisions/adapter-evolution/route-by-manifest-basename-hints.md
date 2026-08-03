---
title: 先按完整 basename 路由再读取文档
status: active
alignment: unaligned
createdAt: 2026-08-03T03:12:20Z
purpose: 以纯 pathname、零新增依赖的确定性匹配先选择 adapter，再进入真实文件处理。
background: 末段 extension 提取不能自然表达复合 suffix，且文件存在性与内容不应成为快速路由的前置条件。
decision: 先匹配 exact filename，再对完整 basename 做大小写归一化 suffix 匹配；命中事实保持私有，选中后才读取并解析。
relations:
  - type: 修订
    target: adapter-evolution/route-by-manifest-pathname-hints.md
---

## 目的

- 让 automatic routing 只依赖调用方提供的 pathname，在任何 document metadata、open、canonicalize、read 或 parse 之前确定是否存在可选 adapter。
- 让普通 extension、复合 suffix 和少量 exact filename 共用一个轻量、确定且由 manifest 拥有的路由模型。
- 保持“快速选择 adapter”与“验证真实格式和内容”两个阶段彼此独立。

## 背景

- Probe traversal 会在 selection 阶段访问文档，并可能与 selected operation 重复读取或解析。
- 只提取最后一段 extension 会把 `.schema.json` 和 `.json` 压成同一粒度，不能表达更具体的复合 suffix 优先级；通用 MIME 表也不能补齐常见 dotfile 和项目专用 suffix。
- Pathname hint 可能与真实内容不一致。把 matched hint 或 format identity 交给 adapter 会让一个低成本、低置信度选择事实渗入真实 parser contract。

## 决策

- 采用: Manifest format descriptor 继续声明 `extensions[]`，并声明可为空的 `filenames[]`；不新增 MIME、content-detection、regex 或 format-inference 依赖，也不建立第二份 alias registry。
- 采用: Automatic routing 从调用 pathname 词法派生 basename；在确定 route 前不得对目标文档执行 metadata、open、canonicalize、read 或 parse。
- 采用: 先以大小写敏感的完整 basename 精确匹配 `filenames[]`；未命中时，把 `extensions[]` 视为带前导点、可包含多个点的 basename suffix，在 ASCII 大小写归一化后与完整 basename 末尾比较。
- 采用: 多个不同长度的 suffix 同时命中时选择最长 suffix；归一化后完全相同的 suffix 仍是 manifest registry 冲突。该语义可用 suffix comparison 实现，不向 manifest 暴露通用 regex、glob 或目录感知 pattern。
- 采用: Pathname match、matched hint 和 matched format identity 只属于 invocation-private selection state，不进入 `StandardOperationInput`、public protocol、ref、continuation 或 typed fields。
- 采用: Route 命中后才执行文件系统路径/访问处理，并由选中 adapter 按自身当时的 grammar、安全规则和 operation contract 读取、解析和验证真实文档；pathname hint 不保证格式真实性。
- 采用: 显式 adapter id 跳过 automatic routing并强制该 adapter 处理真实文档；selected parse、semantic 或 operation failure 不重新路由或 fallback。
- 不采用: Terminal-extension-only extraction、route 前 document I/O、content sniffing、adapter probe traversal、selected failure 后 fallback，以及由本决策规定 adapter 内部 parser/dialect mapping。
