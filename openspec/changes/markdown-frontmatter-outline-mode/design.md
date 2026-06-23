本 design 说明 Markdown frontmatter 如何按 `docnav-markdown` adapter config 中的 enum 策略在 outline 阶段暴露；它只在 `openspec/changes/markdown-frontmatter-outline-mode/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Markdown frontmatter 当前不会进入 outline，这符合 heading model；但 frontmatter 常携带文档级标题、摘要、状态、分类或渲染元数据。调用方在 `outline -> ref -> read` 流程中选择章节前，可能需要先看到这些文档级上下文。

现有 readable-view renderer 的 block config 把声明的 pointer 当成必需字段；如果预留 `/frontmatter/content` 而某些文档没有 frontmatter，或配置选择不暴露 frontmatter，会触发 render failure。因此本 change 同时需要 outline result metadata 扩展和 renderer optional block 能力。

## Goals / Non-Goals

**Goals:**

- Markdown adapter 识别文档开头的 YAML frontmatter delimiter block，但继续把它排除在 heading model 之外。
- `docnav-markdown` adapter config 使用 `options.frontmatter_outline_mode` enum 控制 outline 暴露策略，合法值为 `inline`、`ref` 和 `hidden`，默认值为 `inline`。
- `inline` 模式在 outline success result 顶层返回 `frontmatter: { content_type: "application/yaml", content: string }`；`content` 保留 delimiter 内部的 YAML 原文 payload，不包含起止 delimiter，也不解析为稳定字段。
- `inline` 模式的 frontmatter content 字符预算和续读行为沿用 read content 的分页规则。
- `ref` 模式在 outline entries 中增加 adapter-owned `FM:frontmatter` ref；读取该 ref 时以 `application/yaml` primary content 返回 YAML 原文。
- `hidden` 模式不在 outline 输出中暴露 frontmatter。
- readable-view renderer 支持 optional block pointer：字段不存在时不输出 block、不失败；字段存在且为字符串时正常外置。
- MCP、core 和 shared output 层只承载 adapter 返回的 frontmatter，不解析 YAML。

**Non-Goals:**

- 不定义 YAML schema、字段语义、排序或规范化。
- 不把 frontmatter 中的文本纳入 heading model、heading index 或正文 heading ref grammar。
- 不让 core `docnav` 或 `docnav-mcp` 读取 `docnav-markdown` adapter config。
- 普通 heading/section read 保持 primary content 契约；frontmatter 只通过 outline inline 字段或 `FM:frontmatter` ref 暴露。

## Decisions

1. **frontmatter 的默认暴露点是 outline 顶层字段。**

   `inline` 是默认模式。Markdown outline success result 可包含顶层 `frontmatter: { content_type: "application/yaml", content: string }`。该字段不放入 `entries[]`，因此 heading entries 继续保持 `ref/display` 的紧凑导航形状。

   `content` 是当前 page 可返回的 frontmatter payload slice；完整 payload 来自 frontmatter delimiter 内部，不包含起止 delimiter。没有可识别 frontmatter、配置为 `hidden` 或配置为 `ref` 时，outline result 省略顶层 `frontmatter` 字段。

2. **frontmatter outline 策略由一个 adapter-owned enum 表达。**

   `docnav-markdown` 拥有 `options.frontmatter_outline_mode`，合法值为 `inline`、`ref` 和 `hidden`，默认值为 `inline`。该 key 进入 `docnav-markdown` adapter config schema。Direct CLI 使用既有优先级合并配置和显式参数，并在调用 outline 前把最终选择写入 adapter-owned options；`invoke` 只遵守请求中已经携带的 option 或 adapter 内置默认值。

3. **inline content 的预算沿用 read content 分页规则。**

   `inline` 模式下，frontmatter content 使用 read content 的 Unicode 字符计数、`limit_chars`、page 续读和不切断 Unicode 字符规则。Outline 的顶层 `page` 仍是整个 outline operation 的 continuation。

   Pagination 顺序为：frontmatter inline content 先消费当前 page 的字符预算；frontmatter 当前 page slice 消费完后，剩余预算再用于 heading entries。若 frontmatter payload 单独占满当前 page，`entries` 可以为空且 `page` 指向下一页；调用方用同一 path、mode、limit 和返回的 page 继续 outline。

4. **ref 模式把 frontmatter 暴露为 adapter-owned read ref。**

   `ref` 模式下，outline 在 heading entries 前返回一个 frontmatter entry，ref 固定为 `FM:frontmatter`，display 提供非空的紧凑 frontmatter 摘要或 cost。读取 `FM:frontmatter` 时，read result 的 primary content 为 YAML 原文 payload，`content_type` 为 `application/yaml`，字符预算和续读行为使用普通 read 规则。

5. **输出层支持 optional block，而不是用户动态配置 block pointer。**

   `docnav-readable` 的 renderer config 增加 required/optional 两类 pointer。required pointer 缺失仍失败；optional pointer 缺失时跳过，存在时必须是字符串并按标准 block framing 输出。

   Outline readable-view 为 `/frontmatter/content` 声明 optional block pointer。该字段存在时，header 保留 `frontmatter.content_type`，`frontmatter.content` 位置使用 block 引用；字段不存在时不输出 frontmatter header 或 block。

6. **Markdown adapter 负责 frontmatter 识别和原文保留。**

   Markdown parser 只识别文档开头的 YAML frontmatter delimiter block，保留 delimiter 内部原文 payload，不把 YAML 解析成业务字段。共享 protocol、core CLI、MCP 和 output renderer 都不理解 YAML 语义。

## Risks / Trade-offs

- [Risk] outline schema 新增可选 `frontmatter` 字段会扩大 consumer 解析面。
  Mitigation: 字段可选；`ref`、`hidden` 或无 frontmatter 时 shape 保持只有 entries/page；schema 示例覆盖 inline/ref/hidden 三种路径。

- [Risk] 默认 `inline` 可能让较长 frontmatter 延后 heading entries 出现。
  Mitigation: spec 明确 inline content 使用 read content 分页规则；调用方可通过 `ref` 模式把 frontmatter 变成单独 read，或通过 `hidden` 模式隐藏。

- [Risk] 新增 `FM:frontmatter` ref 可能被误认为 heading ref。
  Mitigation: 文档明确该 ref 是 Markdown adapter-owned frontmatter ref；共享层继续把 ref 当作 opaque string，heading ref grammar 不变。

- [Risk] Optional block 可能掩盖拼错 pointer 的 renderer config。
  Mitigation: optional pointer 只在字段缺失时跳过；字段存在但类型错误、pointer 重复或 identity 冲突仍按 render failure 处理。

- [Risk] frontmatter 解析边界与 Markdown parser 行为不一致。
  Mitigation: adapter 文档和测试只承诺文档开头 delimiter block；未闭合、非文档开头或不符合 delimiter 规则的文本保持普通正文处理。

## Migration Plan

1. 更新 protocol/readable outline schema、Markdown config schema、readable-view renderer config 和 conformance vectors。
2. 更新 Markdown adapter frontmatter extraction、outline inline/ref/hidden result construction、`FM:frontmatter` read handling 和 `frontmatter_outline_mode` option 接入。
3. 更新 Markdown direct CLI smoke、readable-json/protocol-json/MCP 示例和 docs。
4. 运行 schema/example validation、Markdown adapter tests 和 workspace verifier。

默认 `inline` 会改变有 frontmatter 文档的 outline 输出；这是本 change 的目标行为。配置为 `hidden` 时，outline 可恢复为不暴露 frontmatter 的形状。

## Open Questions

- 无。

## 实现边界

- Frontmatter 保留 YAML 原文；实现阶段不得把 frontmatter YAML 解析为稳定字段，除非另开 change 定义字段语义。
- 本 change 的 Current docs、schema 和 examples 只声明已实现的 outline inline/ref/hidden 行为；涉及其它全文读取能力的交接说明只保留在对应 change 中。
