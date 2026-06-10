本 proposal 仅为 `shorten-markdown-refs` change 的未审核临时文档，核心目标是把 Markdown adapter 的 canonical heading ref 完全迁移为短标识，降低 CLI `--ref` 输入长度和复制出错概率。

本 change 只在 `openspec/changes/shorten-markdown-refs/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## Why

当前 Markdown heading ref 把完整 heading breadcrumb 编入 `--ref`，深层标题、长标题和重复路径会让命令行输入显著变长，并提高复制、转义和截断风险。
现在需要把 Markdown adapter 的 canonical ref 迁移为短标识，让 `outline -> ref -> read` 在 CLI-first 场景中更可靠。

## What Changes

- **BREAKING**：Markdown heading canonical ref 从 `L{line}:{path}` / `L{line}#{ordinal}:{path}` 完全迁移为短标识格式。
- **BREAKING**：Markdown `read` 不再接受旧的长 heading ref 格式；调用方必须重新执行 `outline` 或 `find` 获取新 ref。
- Markdown adapter 必须继续保证同一文档内生成的 heading ref 非空、唯一，并可被 `read` 原样消费。
- Markdown `find` 返回的 match ref 必须使用同一套短 heading ref。
- 全文 fallback ref 是否改名由 design 明确；无论选择如何，必须在 specs 中固定为 adapter 自有契约。
- 不引入 `--like`、fuzzy resolve、core ref 解析或 MCP 特殊兜底；这些属于后续独立 interface change。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter-v0-implementation`：修改 Markdown heading ref 的 canonical 格式、读取解析规则、find ref 归属输出，以及相关测试验收。

## Impact

- 影响 `docnav-markdown` adapter 的 ref 生成、ref 解析、outline/find/read 输出和相关测试 fixture。
- 影响通过 `docnav` core、MCP 或脚本传递 Markdown ref 的可观察输出，但 core、MCP 和共享协议仍只原样传递 adapter 返回的 ref。
- 影响文档、schema 示例、golden output 和 smoke 测试中包含旧 Markdown heading ref 的材料。
- 不改变 protocol result 字段 shape、错误 code、page 模型、`Entry { ref, display }` shape 或 core adapter routing。
