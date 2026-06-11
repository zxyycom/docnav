本 change 仍处于审核阶段，不代表现行规范或已确认实现方案。

## Why

当前 Markdown heading ref 将完整 breadcrumb 编入 ref。ref 长度因此随标题长度和层级深度增长，增加 CLI 传递、复制和转义成本。

Markdown adapter 需要一种不包含标题内容的定位格式，使 `outline -> ref -> read` 的成本主要取决于文档结构规模，而不是文档文本长度。

## What Changes

- **BREAKING**：Markdown heading ref 改为 `H:L{line}:H{level}:I{index}`；`read` 不再接受旧 `L{line}:{path}`、`L{line}#{ordinal}:{path}` 和显式 `L{line}#1:{path}`。
- `line` 表示 1-based 起始行号，`level` 表示 Markdown heading level，`index` 表示该 heading 在全文有效 headings 中的 1-based 顺序号。
- `L`、第二个 `H` 和 `I` 分别标识 line、heading level 和全文 index，调用方无需依赖字段顺序记忆数字含义。
- Markdown adapter 在同一次文档解析结果中生成非空、唯一且可由 `read` 消费的 heading ref。文档内容变化后，调用方重新执行 `outline` 或 `find`。
- Markdown adapter 保留私有 ref `doc:full`，用于读取整篇 Markdown 文档。
- 新增 `docs/adapters/markdown.md`，集中说明 Markdown 的 outline、read、find、ref、整篇文档读取行为、保证范围和验证入口。
- 共享 ref 文档只定义 adapter 所有权和不透明值原样传递，不承载任何 adapter 私有 ref 语义。

## Capabilities

### New Capabilities

- 无。

### Modified Capabilities

- `markdown-adapter-v0-implementation`：修改 Markdown heading ref 的格式、生成、读取和验收规则。
- `v0-contract-documentation`：收敛共享 ref 契约，并明确 adapter 专属文档的规则所有权。

## Impact

- 修改 `docnav-markdown` 的 heading ref 生成与解析，以及 outline/find/read 的可观察输出。
- 更新包含 Markdown heading ref 的规范、文档、示例、fixture、golden output 和测试。
- 放宽主规范中面向所有 adapter 的 ref 唯一性、消歧和文档变化要求。
- 不改变 protocol result shape、错误码、分页模型、core adapter routing 或 MCP 映射职责。
