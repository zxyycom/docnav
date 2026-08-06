---
title: 使用 manifest pathname hints 选择 adapter
status: archived
alignment: null
createdAt: 2026-07-31T02:54:31Z
purpose: 以无文件 I/O、无 registry 顺序依赖的方式快速选择唯一 adapter。
background: Probe traversal 重复读取文档；通用 MIME 表不能覆盖常见 dotfile 与项目专用文件名。
decision: 自动路由只用 manifest extension/exact-filename hints；不加识别依赖，内容由选中 adapter 验证。
relations: []
---

## 目的

- 让 automatic routing 在进入 adapter 前只做一次确定性的 pathname lookup。
- 让 adapter manifest 成为 format identity 与 pathname routing metadata 的唯一 owner。
- 把“快速选择 adapter”与“验证文档是否符合格式”保持为两个独立阶段。

## 背景

- Current probe traversal 会按 registry 顺序调用 adapter-owned probe，并可能在 selection 和 operation 阶段重复读取、解码或解析同一文档。
- Dotfile（例如 `.prettierrc`）没有普通 extension；项目专用 suffix（例如 `.code-workspace`）也不一定存在于通用 MIME 表。
- 调查中的 `mime_guess` 虽然较轻且成熟，但对上述样本不能直接给出可用映射。为补齐映射仍要维护项目自有 alias，因而外部依赖不会减少 Docnav 的 metadata ownership。
- 本决策先于 `replace-probe-traversal-with-inferred-routing` 的实现生效；在该 change 完成前，Current 主规范和代码仍可能描述 probe，所以 alignment 为 `unaligned`。

## 决策

- 采用: 每个 manifest format descriptor 继续声明 `extensions[]`，并新增可为空的 `filenames[]` exact-basename hints。
- 采用: Automatic routing 不读取文件。它先以大小写敏感的 basename 精确匹配 `filenames[]`；没有 filename 命中时，再以 ASCII 大小写不敏感方式匹配末尾 `extensions[]`。
- 采用: Exact filename 优先于通用 extension，使特定配置文件可以覆盖其 suffix 的默认 route。
- 采用: Pathname hint 只决定 adapter selection，不证明格式真实性。选中的 adapter 必须读取并按自身 then-current grammar 与安全规则验证文档。
- 采用: 一旦选中 adapter，其 parse、semantic 或 operation failure 原样返回，不重新路由或尝试其他 adapter。
- 采用: 显式 adapter id 跳过 automatic routing，精确选择对应 adapter 并强制由它解析真实文档。
- 采用: Format identity、同类 extension hint 和同类 filename hint 在 validated registry 中必须唯一；construction、doctor 与 release validation 阻断冲突，runtime 只保留防御性 global invariant failure。
- 采用: Routing implementation 不增加 MIME、content-detection 或 format-inference 依赖，也不建立第二份 alias registry、confidence scoring 或 detector extension point。
- 不采用: Adapter probe traversal、content sniffing、MIME table 作为默认 automatic route，以及 selected failure 后的 fallback。
- 边界: 具体 alias 只表示当时希望快速选择的 adapter；JSONC、JSON5、NDJSON 等 grammar 或 document-model 支持必须由各自 owner/change 另行决定，不能从 pathname route 推导。
