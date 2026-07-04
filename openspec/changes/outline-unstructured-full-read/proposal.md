本 change 让 navigation input resolution 通过配置 selectors 产出 `outline_mode = "unstructured_full"` 后，`outline` 在 adapter 正常 outline dispatch 前直接返回全文内容。

## Why

有些文档不适合通过 heading、outline entry 或局部 ref 阅读。对这类文档继续强制 `outline -> ref -> read` 会让调用方先拿到低价值 ref，再多发一次 `read`。当 path rule 显式选择全文读取，或 selected adapter 的 cost threshold 判定全文成本足够低时，第一次 `outline` 应直接返回原文全文。

## What Changes

- `outline` success result 改为带 `kind` 的 union：structured branch 保留 entries/page 语义，unstructured branch 直接返回 content/content_type/cost/reason。
- `outline_mode` 是 navigation-owned execution policy；resolution 顺序为 path rules > adapter-scoped cost thresholds > built-in default。
- Cost threshold evaluation 先按 selected adapter 过滤并按 unit 合并最小阈值，再按需调用 adapter cost measurement hook。
- Adapter contract 增加非结构化全文 hook set，用于格式专属全文读取、cost measurement 和稳定 result facts 补充。
- Non-structured full-read result 不返回 entries/ref/page/continuation，不分页、不裁剪，也不新增 public CLI outline-mode override flag。

## Capabilities

- `navigation-input-resolution`: 新增 `outline_mode`、path rules、adapter-scoped cost threshold selector 和 pre-dispatch resolution 边界。
- `adapter-protocol`: 扩展 outline result union，并新增非结构化全文 hook set。
- `docnav-contracts`: 记录 `outline -> ref -> read` 主流程外的 opt-in full-read exception。
- `core-cli`: 映射标准 `outline_mode` 的 readable/protocol 输出，不新增 override flag。
- `markdown-navigation`: 覆盖 Markdown 在 navigation-level `unstructured_full` 下的 smoke 行为，并保持正常 outline 不变。
- `readable-view-output`: 支持非结构化 outline 的 `/content` block。

## Impact

受影响范围：navigation input resolution、outline execution pipeline、adapter hook contract、shared protocol/result types、readable output mapping、config schema/examples 和 Markdown smoke fixtures。

不受影响范围：默认 structured outline 行为、read/find/info、ref opaque pass-through、Markdown heading ref grammar 和 adapter 私有导航策略。
