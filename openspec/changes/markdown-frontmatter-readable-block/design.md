本 design 说明 Markdown frontmatter 如何作为可选 metadata block 随普通 read 输出；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

Markdown frontmatter 当前不会进入 outline，这符合 heading model；但 frontmatter 常携带文档级标题、摘要、状态、分类或渲染元数据。调用方只读取某个 heading section 时，可能缺少这些文档级上下文。

现有 readable-view renderer 的 block config 把声明的 pointer 当成必需字段；如果预留 `/frontmatter/content` 而某些文档没有 frontmatter，会触发 render failure。因此本 change 同时需要 read result metadata 扩展和 renderer optional block 能力。

## Goals / Non-Goals

**Goals:**

- Markdown adapter 识别 YAML frontmatter，但继续把它排除在 heading outline 之外。
- Markdown read 可在生效配置或显式 adapter option 启用时随普通 heading/section read 返回 frontmatter metadata。
- frontmatter metadata 保留 YAML 原文和 `application/yaml` content type，不解析为稳定字段。
- readable-view renderer 支持 optional block pointer：字段不存在时不输出 block、不失败；字段存在且为字符串时正常外置。
- MCP 和 core 只承载 adapter 返回的 metadata，不解析 YAML。

**Non-Goals:**

- 不把 frontmatter 转成 outline entry 或 read ref。
- 不定义 YAML schema、字段语义、排序或规范化。
- 不在 `doc:full` 或非结构化全文读取中默认重复输出 frontmatter。
- 不改变普通 read content 的分页和字符预算。

## Decisions

1. **frontmatter 属于 read metadata，不属于 outline。**

   Metadata shape 使用 `frontmatter: { content_type: "application/yaml", content: string }` 或等价 typed field。它只在 read 成功结果中出现，不生成 ref，不参与 find match 归属。

   备选方案是生成 `doc:frontmatter` ref。该方案可独立读取 metadata，但仍需要额外调用，不能解决“读章节时遗漏上下文”的问题。

2. **输出层支持 optional block，而不是用户动态配置 block pointer。**

   `docnav-readable` 的 renderer config 增加 required/optional 两类 pointer。required pointer 缺失仍失败；optional pointer 缺失时跳过，存在时必须是字符串并按标准 block framing 输出。

   备选方案是让 frontmatter content 留在 JSON header。该方案会让多行 YAML 破坏 readable-view 的信息密度和可审计性。

3. **Markdown adapter 负责 frontmatter 识别和原文保留。**

   Markdown parser 只识别文档开头合法 YAML frontmatter block，保留原文 payload，不把 YAML 解析成业务字段。共享 protocol、core CLI、MCP 和 output renderer 都不理解 YAML 语义。

4. **配置只决定普通 read 是否附带 frontmatter。**

   `docnav-markdown` 拥有一个 adapter-owned frontmatter 输出开关。这个 change 只定义开关语义：启用时普通 read 可以额外返回 frontmatter metadata，未启用时省略；具体配置文件、格式和合并方式由配置能力或对应主规范定义。Direct CLI 或上游调用方在调用 Markdown read 前把最终选择显式化为 adapter-owned option 或等价语义输入；`invoke` 只遵守请求中已经携带的 option。

5. **全文原文读取不重复 metadata。**

   当 read 目标是 `doc:full` 或 outline-unstructured-full-read 的全文内容时，frontmatter 已经包含在原文中，默认不再额外输出 metadata block。后续若需要“正文剥离 frontmatter、metadata 单独 block”，应另开 change。

## Risks / Trade-offs

- [Risk] 协议和 readable schema 新增可选 metadata 字段会扩大 consumer 解析面。  
  Mitigation: 字段可选；未启用配置或无 frontmatter 时 shape 保持旧结果；schema 示例覆盖有/无 metadata 两种路径。

- [Risk] Optional block 可能掩盖拼错 pointer 的 renderer config。  
  Mitigation: optional pointer 只在字段缺失时跳过；字段存在但类型错误、pointer 重复或 identity 冲突仍按 render failure 处理。

- [Risk] frontmatter 解析边界与 Markdown parser 行为不一致。  
  Mitigation: adapter 文档和测试只承诺文档开头 YAML frontmatter；非法或非 YAML frontmatter 保持普通正文处理。

- [Risk] section read 的字符预算不包含 frontmatter 可能让输出总量超过预期。  
  Mitigation: spec 明确 read content 预算仍只约束 primary content；frontmatter metadata 是可选上下文 block，由生效配置或显式 option 启用。

## Migration Plan

1. 更新 protocol/readable schema、readable-view renderer config 和 conformance vectors。
2. 更新 Markdown adapter frontmatter extraction、read result metadata 和配置开关接入。
3. 更新 Markdown direct CLI smoke、readable-json/protocol-json/MCP 示例和 docs。
4. 运行 schema/example validation、Markdown adapter tests 和 workspace verifier。

默认不开启时，既有用户输出不变。

## Open Questions

- 无。

## 实现边界

- Frontmatter metadata 保留 YAML 原文；实现阶段不得把 frontmatter YAML 解析为稳定字段，除非另开 change 定义字段语义。
