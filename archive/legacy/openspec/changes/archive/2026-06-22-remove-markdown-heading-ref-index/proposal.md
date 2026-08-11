本 change 定义 Markdown heading canonical ref 使用 line + level 结构坐标，并同步 adapter、文档、示例和测试边界。

## Why

Docnav 将 Markdown ref 定义为生成时解析结果中的结构坐标，主流程是 `outline -> ref -> read`。line + level 已提供当前解析结果内的 heading 定位输入，并保持 ref 短小、可读、便于人工审计。

使用 line + level 可以让 Markdown ref 更贴近 CLI-first 阅读场景：调用方仍从 outline/find 获取 ref 并原样传给 read，adapter 仍按自身私有 grammar 解释 ref，共享层继续把 ref 当作 opaque string。

## What Changes

- **BREAKING**: Markdown heading canonical ref 使用 `H:L{line}:H{level}`。
- **BREAKING**: `docnav-markdown read` 对 heading ref 的匹配字段为 line 和 level；符合当前 canonical grammar 且当前解析结果缺少匹配 heading 时返回 `REF_NOT_FOUND`。
- 更新 Markdown adapter 文档、OpenSpec requirement、docs/examples、readable/protocol/MCP 示例和测试断言中出现的 canonical heading ref。
- `doc:full` 继续作为 Markdown adapter 私有全文 ref，全文读取语义保持稳定。
- 共享 ref 契约保持稳定：`docnav` core、MCP 和共享协议校验 ref 是非空字符串，并将 ref 原样传给 adapter。
- 范围边界：title、breadcrumb、hash、文档版本和持久身份保持在 display、content 或外部状态中；其它 adapter 的 ref grammar、共享层 opaque pass-through 职责保持稳定。

## Capabilities

### New Capabilities

- 无新增 capability。

### Modified Capabilities

- `markdown-navigation`: 修改 Markdown heading ref grammar、read 匹配规则、错误边界和相关验证材料。

## Impact

- 受影响 public surface：Markdown adapter 的 `outline`、`find` 输出 ref，`read --ref` 接受的 canonical heading ref，readable/protocol/MCP 示例中展示的 Markdown heading ref，以及 `REF_INVALID`/`REF_NOT_FOUND` 边界。
- 受影响代码：`crates/docnav-markdown` 的 heading ref 生成、解析和匹配逻辑，以及依赖该 ref 字符串的 adapter/unit/CLI smoke 测试。
- 受影响文档与验证材料：`docs/adapters/markdown.md`、`docs/examples/**`、相关 schema/example 索引说明、OpenSpec `markdown-navigation` delta spec 和测试用例维护材料。
- 保持稳定范围：共享 ref opaque contract、`docnav` core adapter routing、MCP bridge 的 pass-through 职责、protocol envelope shape、pagination 字段和其它 adapter。
