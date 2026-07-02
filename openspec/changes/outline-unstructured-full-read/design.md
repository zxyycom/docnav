本 design 说明配置命中的非结构化文档如何在 `outline` 时直接返回全文内容；它只在 `openspec/changes/outline-unstructured-full-read/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Context

当前 Docnav 把 `outline` 定义为扁平 `entries[]` 和 `page`，调用方再把 entry ref 原样传给 `read`。这个流程适合 heading、章节、书签等结构化文档，但不适合被项目显式标记为“不要结构化导航”的文档。

对这类文档继续返回 `doc:full` 或其它全文 ref 只会让模型在无法预知内容的情况下多调用一次 `read`。本 change 将该路径定义为 opt-in 的 `outline` 结果形态扩展：配置命中时直接读取原文全文。

## Goals / Non-Goals

**Goals:**

- 让 `docnav outline <path>` 在非结构化配置命中时一次性返回完整原文内容。
- 保证该路径不返回 ref、不返回 page、不提供 continuation。
- 保持未命中配置的普通文档继续使用既有结构化 `outline -> ref -> read` 流程。
- 让 core CLI 与 linked Markdown adapter 对同一生效配置语义保持一致。

**Non-Goals:**

- 不改变普通结构化 outline 的 entries、display、ref、page 和分页规则。
- 不把 `ignore` 解释为跳过文件或 adapter probe。
- 不引入非结构化全文读取分页、摘要、chunking 或自动结构恢复。
- 不要求共享层解析 Markdown heading、frontmatter 或任何格式私有结构。

## Decisions

1. **使用 `OutlineResult` union，而不是复用 `doc:full`。**

   命中非结构化配置后，`outline` 返回 `kind: "unstructured"`、`reason: "configured_unstructured_document"`、`content`、`content_type` 和 `cost`。结构化结果继续返回 `kind: "structured"` 或等价可判别字段、`entries` 和 `page`。

   备选方案是继续返回 `doc:full` ref。该方案保持旧 shape，但会强制调用方额外调用 `read`，与本 change 的目标冲突。

2. **非结构化全文读取不分页。**

   配置命中时，`page` 和 continuation 不出现在成功结果中；`limit` 和 `page` 仍可由 CLI 参数解析层校验，以保持命令行兼容，但不用于裁剪全文内容。

   备选方案是按 read 的 `limit` 分页。该方案更节省单次输出，但会重新引入 continuation 与多轮调用，违背“自动全文阅读”的用户意图。

3. **配置命中是 execution policy，不是 adapter ref policy。**

   本 change 只定义 `outline` 在获得生效非结构化文档策略后如何执行：命中后按文本文件原文读取并走 document output pipeline。具体配置文件、格式和合并方式由配置能力或对应主规范定义；共享 ref 契约不新增全文 ref。

   备选方案是让 Markdown adapter 在 outline 中生成特殊 ref。该方案不能减少模型调用，也会把 execution policy 藏进 adapter ref。

4. **readable-view 为非结构化 outline 单独声明 `/content` block。**

   普通 outline 仍是无 block header；非结构化 outline 使用 content block，避免把完整原文塞进 JSON header。renderer config 仍是仓库内代码契约，不由用户配置动态控制。

5. **strict output projection 使用 documented success payload。**

   配置命中的非结构化 outline 是成功执行策略，readable-json 使用 documented outline success payload 分支，protocol-json 包装同一 success result，readable-view 只使用 `/content` block 表达正文。未命中配置时继续保留 Markdown `doc:full` fallback navigation behavior；两者都不通过 primary diagnostic wrapper 表达。



## Risks / Trade-offs

- [Risk] `outline` 不再总是 entries/page，现有消费者若假设固定 shape 会失败。  

- [Risk] 全文读取可能输出很大内容。  
  Mitigation: 这是配置显式选择的行为；实现和 docs 必须说明该路径不使用 `limit_chars` 分页，配置应只用于用户确认适合全文阅读的文档。

- [Risk] 本 change 过早定义配置来源，后续可能与 core 配置模型冲突。
  Mitigation: 本 change 只引用“生效配置”语义；具体配置文件、格式和合并方式留给配置能力 change 或主规范沉淀。

- [Risk] 非结构化结果与普通 outline 共用 operation 名称，阅读输出文案可能误导调用方。  
  Mitigation: readable payload 必须包含稳定 `kind`、`reason` 或等价字段，并在 readable-view header 中说明自动全文读取。

## Migration Plan

1. 先更新主规范、schema、examples 和 OpenSpec delta 中的 union shape。
2. 再实现 shared protocol/readable result 类型和 readable-view config。
3. 接着在 core outline 入口和 linked Markdown adapter 接入生效非结构化策略。

未配置非结构化规则的项目无需迁移。

## Open Questions

- 无。

## 实现边界

- 非结构化 outline 的主要结果是全文内容 branch；实现阶段不得重新引入 `doc:full` ref 作为该配置路径的主要结果。
