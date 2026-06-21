本 change 目标是移除 Markdown heading ref 中的 `I{index}` 字段，让 canonical heading ref 只由行号和 heading level 构成；它只在 `openspec/changes/remove-markdown-heading-ref-index/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 Markdown canonical heading ref 使用 `H:L{line}:H{level}:I{index}`，其中 `I{index}` 主要提供文档结构变化后的额外失效能力。Docnav 已将 Markdown ref 定义为生成时解析结果中的结构坐标而非持久身份，因此该字段相对主流程 `outline -> ref -> read` 过度设计，也增加了 ref 长度和人工审计成本。

现在去掉 index 可以让 Markdown ref 更贴近 CLI-first 阅读场景：调用方仍从 outline/find 获取 ref 并原样传给 read，adapter 仍按自身私有 grammar 解释 ref，共享层继续把 ref 当作 opaque string。

## What Changes

- **BREAKING**: Markdown heading canonical ref 从 `H:L{line}:H{level}:I{index}` 改为 `H:L{line}:H{level}`。
- **BREAKING**: `docnav-markdown read` 对 heading ref 的匹配字段从 line、level、index 改为 line、level；符合新 canonical grammar 但当前解析结果无匹配 heading 时返回 `REF_NOT_FOUND`。
- 更新 Markdown adapter 文档、OpenSpec requirement、schema/example/fixture、readable/protocol/MCP 示例和测试断言中出现的 canonical heading ref。
- 保留 `doc:full` 作为 Markdown adapter 私有全文 ref，不改变全文读取语义。
- 保留共享 ref 契约：`docnav` core、MCP 和共享协议仍只校验 ref 是非空字符串，并将 ref 原样传给 adapter。
- 非目标：不为 Markdown ref 引入 title、breadcrumb、hash、文档版本或持久身份；不改变其它 adapter 的 ref grammar；不要求共享层解析 Markdown ref。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-navigation`: 修改 Markdown heading ref grammar、read 匹配规则、错误边界和相关验证材料。

## Impact

- 受影响 public surface：Markdown adapter 的 `outline`、`find` 输出 ref，`read --ref` 接受的 canonical heading ref，readable/protocol/MCP 示例中展示的 Markdown heading ref，以及 `REF_INVALID`/`REF_NOT_FOUND` 边界。
- 受影响代码：`crates/docnav-markdown` 的 heading ref 生成、解析和匹配逻辑，以及依赖该 ref 字符串的 adapter/unit/CLI smoke 测试。
- 受影响文档与验证材料：`docs/adapters/markdown.md`、`docs/examples/**`、相关 schema/example 索引说明、OpenSpec `markdown-navigation` delta spec 和测试用例维护材料。
- 不受影响范围：共享 ref opaque contract、`docnav` core adapter routing、MCP bridge 的 pass-through 职责、protocol envelope shape、pagination 字段和非 Markdown adapter。
