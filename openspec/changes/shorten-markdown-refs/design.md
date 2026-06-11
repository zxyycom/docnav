本 design 记录当前候选方案，仍需审核后才能进入实现。

## Context

ref 由格式 adapter 生成和消费；共享协议、`docnav` core 和 MCP 只负责原样传递。

当前 Markdown heading ref 使用 `L{line}:{path}` 或 `L{line}#{ordinal}:{path}`。完整标题路径（breadcrumb）使 ref 长度受标题文本控制。为 heading 内容生成摘要会额外引入算法、编码、规范化和碰撞语义，但不能提供绝对的内容身份保证。

本方案改用 Markdown parser 已有的结构位置，不承担文档版本或内容身份校验。

## Goals / Non-Goals

**Goals:**

- heading ref 不包含标题、breadcrumb 或其摘要。
- 同一次解析结果中的 heading ref 非空、唯一，并可由 `read` 消费。
- outline 和 find 对同一 heading 生成相同 ref。
- Markdown 私有导航行为由独立 adapter 文档拥有。

**Non-Goals:**

- 文档版本、mtime、内容 hash 和过期 ref 检测。
- 保持 heading ref 跨文档修改或 parser 版本稳定。
- 在 core 或 MCP 中解析、转换或兼容 Markdown ref。

## Decisions

### 1. Heading ref 使用 `H:L{line}:H{level}:I{index}`

- 首个 `H`：标识 heading ref 类型。
- `L{line}`：heading 的 1-based 起始行号。
- `H{level}`：Markdown heading level，范围为 `1` 到 `6`。
- `I{index}`：heading 在全文有效 headings 中的 1-based 顺序号。

三个数字字段均使用不带前导零的十进制表示。canonical 格式等价于正则 `^H:L([1-9][0-9]*):H([1-6]):I([1-9][0-9]*)$`。`index` 在 `max_heading_level` 等可见性过滤前确定，因此 outline 和 find 不会因过滤参数不同而重新编号同一 heading。

字段标识使每个数字的含义可直接识别。ref 长度仅随行号和 heading 数量的位数增长，不随标题长度、breadcrumb 深度或字符集增长。

未采用的方案：

- 内容摘要：增加实现和契约复杂度，但不能消除碰撞或文档变化问题。
- 纯 index：更短，但缺少用于人工审计的行号和 heading level。
- line + level：字段较少，但缺少明确的全文顺序标识。

### 2. Read 精确匹配三个字段

`read` 只接受 canonical 标记格式，并在当前解析结果中匹配 line、level 和 index 全部相同的 heading。没有匹配项时返回 `REF_NOT_FOUND`。

该过程只使用结构位置，不比较 heading title 或 section 内容。ref 的有效上下文是当前文档内容；文档变化后，由调用方重新执行 `outline` 或 `find` 获取新 ref。

### 3. `doc:full` 属于 Markdown adapter

当 outline 参数下没有可见 heading 时，Markdown adapter 返回单条 `doc:full` entry。`read` 使用该 ref 返回整篇 Markdown 文档。

`doc:full` 只由 Markdown adapter 的规范和文档定义。共享 ref 契约不建立对应的通用 ref 类型。

### 4. 旧 heading ref 直接迁移

新版本只生成和接受 `H:L{line}:H{level}:I{index}` heading ref。持有旧 heading ref 的调用方通过重新执行 `outline` 或 `find` 完成迁移。

core 和 MCP 不参与格式转换，继续原样传递 ref。

### 5. Markdown 行为使用独立主文档

新增 `docs/adapters/markdown.md`，作为当前 Markdown adapter 行为的长期主文档，覆盖：

- heading 识别、outline 可见性和 section 范围；
- heading ref 的生成和读取；
- `doc:full` 的生成条件和读取行为；
- find match 的 ref 归属；
- ref 的保证范围和调用方责任；
- Markdown 默认值和验证入口。

`docs/refs.md` 只保留共享边界并链接到 adapter 专页。`docs/references/markdown-navigator.md` 只记录外部来源和迁移依据，不拥有现行行为规则。

## Risks / Trade-offs

- 旧 Markdown heading ref 在迁移后失效。调用方需要重新执行 `outline` 或 `find`。
- 文档或 parser 结果变化可能使数字 ref 失效或指向新的结构位置。该行为不属于 Markdown adapter 的稳定性保证。
- 不同 adapter 可以定义不同的 ref 行为。每个已实现 adapter 通过自己的文档说明具体契约。
- ref 示例分散在文档和测试材料中。实施时通过全仓搜索和 workspace 验证同步更新。

## Migration Plan

1. 确认本 change 的 ref 格式、Markdown 主文档和共享边界。
2. 更新 Markdown heading ref 的生成、解析和 find 归属。
3. 新增 `docs/adapters/markdown.md`，并从共享文档和参考文档迁移现行 Markdown 行为。
4. 更新规范、示例、fixture、golden output 和测试。
5. 放宽主规范中的全局 ref 保证，保留 core/MCP 将 ref 作为不透明值原样传递的边界。
6. 运行局部验证和 `pnpm run verify:docnav-workspace`。

回退时整体恢复旧 Markdown heading ref 的实现和验收材料，不在 core 或 MCP 中增加格式兼容逻辑。
