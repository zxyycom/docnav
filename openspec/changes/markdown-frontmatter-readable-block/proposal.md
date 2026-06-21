本 change 目标是让 Markdown frontmatter 可按配置作为可选 metadata block 随普通 read 输出，减少 heading section 阅读时遗漏文档级元数据的风险；它只在 `openspec/changes/markdown-frontmatter-readable-block/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

Markdown frontmatter 是文档级元数据，不应进入 heading outline，但它常包含标题、摘要、分类、状态或渲染配置；当调用方只读取某个 heading section 时，这些必要上下文容易被遗漏。

现在需要把 frontmatter 作为明确的 Markdown adapter-owned metadata 暴露给 read 输出，并让 readable-view renderer 支持“有内容才显示”的可选 block。

## What Changes

- Markdown parser 识别文档开头 YAML frontmatter，并继续保证 frontmatter 不作为 heading entry 进入 outline。
- Markdown read 在生效配置或显式 adapter option 启用、且当前读取不是全文原文时，可返回 optional frontmatter metadata，包括 `content_type: application/yaml` 和 YAML 原文内容。
- 共享协议/readable payload 扩展 read success result，支持可选 metadata/frontmatter 字段；没有 frontmatter 或配置未启用时字段省略。
- readable-view renderer config 增加 optional block pointer 能力：配置声明的可选 block 字段不存在时不报错，存在且为字符串时按普通 block framing 输出。
- Markdown adapter/direct CLI 增加 frontmatter 输出开关语义，配置只控制是否随普通 read 附带 frontmatter metadata；具体配置文件、格式和合并方式不由本 change 定义。
- 非目标：不把 frontmatter heading 化；不让 core、MCP 或 shared output 层解析 YAML；不默认在非结构化全文读取或 `doc:full` 原文读取中重复输出 frontmatter；不定义 YAML 语义字段解析，只保留原文 metadata block。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `adapter-protocol`: 扩展 read success result，使 adapter 可以返回可选 metadata/frontmatter 内容。
- `docnav-contracts`: 更新协议、readable schema/example、MCP mapping 和 adapter ownership 边界，声明 frontmatter 由 Markdown adapter 解析并由共享层原样承载。
- `markdown-navigation`: 增加 Markdown frontmatter 识别、配置、read metadata 输出和测试边界。
- `readable-view-output`: 增加 optional block pointer 契约，使 `/frontmatter/content` 等可选 metadata block 在字段缺失时不触发 render failure。

## Impact

- 受影响 public surface：Markdown read 的 protocol/readable/MCP 成功结果、readable-view block framing、readable-json schema、MCP `document_read` structuredContent、Markdown frontmatter 配置语义和示例。
- 受影响代码：Markdown parser/frontmatter extraction、Markdown read result construction、shared protocol/readable result types、readable renderer config validation 和 block extraction、schema/example validation。
- 受影响文档与验证材料：`docs/protocol.md`、`docs/output.md`、`docs/mcp.md`、`docs/adapters/markdown.md`、`docs/schemas/**`、`docs/examples/**`、readable conformance vectors 和 Markdown CLI smoke fixtures。
- 不受影响范围：outline heading 可见性、Markdown heading ref grammar、普通 read content 字符预算、`doc:full` 原文读取默认内容、非 Markdown adapter 的 metadata 策略。
