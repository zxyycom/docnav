# Markdown Adapter

本文是 `docnav-markdown` 当前实现的导航行为和私有契约主文档。它覆盖 Markdown adapter 的 outline、read、find、ref grammar、结构快照语义、`doc:full`、错误分类、保证范围和验证入口，是 Markdown 实现与审计的长期规范来源。

共享协议、`docnav` core 和 MCP 只按共享 ref 契约原样传递 ref，不解析 Markdown heading ref 内部结构。共享 ref 最小契约见 [Ref](../refs.md)。

## Heading 识别与 Section 范围

Markdown adapter 使用成熟 parser 识别有效 heading（H1–H6）。section 范围从目标 heading 开始，包含更深层 heading，在下一个同级或更高级 heading 前结束。

不进入 outline 的内容：

- frontmatter。
- 代码围栏内的伪 heading。

## Outline

Markdown outline 按文档顺序返回扁平 heading entries。每条 entry 包含：

| 字段 | 内容 |
| --- | --- |
| `ref` | `H:L{line}:H{level}:I{index}`，不含 title 或 breadcrumb |
| `display` | heading title 或 breadcrumb 导航文本，可含 heading level、section cost 等紧凑摘要 |

层级关系只通过 display 中的 heading title 或 breadcrumb 表达，不生成通用树字段。

### 可见性与 `max_heading_level`

Markdown adapter 内置 `max_heading_level: 3`（可由 `docnav-markdown` 配置域覆盖）。可见性过滤只影响 outline 返回哪些 heading entries，不改变 `index` 的分配。`index` 在过滤前基于全文有效 headings 确定，保证同一解析结果中的同一 heading 在 outline 和 find 中使用相同 ref。

### 无可见 heading 时的全文 ref

当前 outline 参数过滤后没有可见 heading 时，outline 返回单条 `doc:full` entry。该 ref 读取整篇 Markdown 文档。

## Find

Markdown find 搜索全文，但 match ref 指向当前 outline 参数下离命中位置最近的 heading entry。最近 heading 按源码位置判断：命中位于两个 outline entry 之间时选择距离更近的一项，距离相同则选择前一项。若当前 outline 参数没有任何 entry，find 使用 `doc:full`。

每个 find match 的 display 保留匹配位置附近的非空文本片段，并可补充对应 heading 的 title 或 breadcrumb。ref 由独立字段完整承载，不包含匹配片段或导航文本。

## Read

### Heading Ref 读取

Markdown `read` 解析 `H:L{line}:H{level}:I{index}` canonical heading ref，并在当前文档解析结果中精确匹配 line、level 和 index 全部相同的 heading。匹配成功时返回该 heading 的当前 Markdown section，`content_type` 为 `text/markdown`。

匹配条件严格限定为三个结构字段，不使用 heading title、breadcrumb、section 内容或其摘要补充匹配。

### `doc:full` 读取

`read` 接受 `doc:full` 并返回整篇 Markdown 文档。`doc:full` 是 Markdown adapter 私有 ref，不属于 heading ref grammar。

## Heading Ref Grammar

Markdown heading ref 使用以下 canonical 格式：

```text
H:L{line}:H{level}:I{index}
```

| 字段 | 含义 | 约束 |
| --- | --- | --- |
| `H` | heading ref 类型标识 | 固定前缀 |
| `L{line}` | heading 的 1-based 起始行号 | 十进制正整数，无前导零 |
| `H{level}` | Markdown heading level | `1`–`6` |
| `I{index}` | 全文有效 headings 中的 1-based 顺序号 | 十进制正整数，无前导零，可见性过滤前确定 |

Canonical grammar 正则：`^H:L([1-9][0-9]*):H([1-6]):I([1-9][0-9]*)$`。

### 长度保证

ref 长度不受 heading title、breadcrumb 深度和 Unicode 文本影响，只随 line 和 heading 数量的十进制位数增长。字段前缀（`L`、`H`、`I`）提供基础语义提示，支持日志和人工审计。

### 同一解析结果中的唯一性

当前 Markdown parser 为每个有效 heading 分配唯一 index，因此 canonical ref 在同一次解析结果中唯一。该唯一性是 Markdown adapter 的私有选择，不属于共享 ref 保证。其它 adapter 无需提供相同保证。

### 重复 Heading

文档包含重复 title 或重复 breadcrumb 时，每个 heading 根据自身 line、level 和 index 获得不同 ref。read 不使用 title 或 breadcrumb 消歧。

## 结构快照语义

Markdown heading ref 是生成时解析结果中的结构坐标，**不是** heading title、section 内容或文档版本的持久身份。

文档内容或 parser 结果变化后，同一个格式合法的 ref 可能：

- 不再匹配任何 heading，返回 `REF_NOT_FOUND`；
- 匹配当前结构中坐标相同的另一个 heading；
- 在结构坐标未变化时继续匹配当前 heading。

以上结果均符合本 adapter 的契约。调用方应在需要当前结构时重新执行 outline 或 find 获取当前 ref。规范不要求调用方预先检测文档是否变化，也不保证旧 ref 一定失败。

## 错误分类

Markdown adapter 在 `read` 中按以下边界映射 ref 错误：

### `REF_INVALID`

输入 ref 是非空字符串，但不符合 Markdown adapter 当前 ref grammar（既不是 canonical heading ref，也不是 `doc:full` 或其它 Markdown 合法 ref）时，返回 `REF_INVALID`。

- 稳定 details 包含原始 `ref` 和非空 `reason`。

### `REF_NOT_FOUND`

输入 ref 符合 canonical heading grammar，但当前解析结果中没有 line、level 和 index 全部匹配的 heading 时，返回 `REF_NOT_FOUND`。

### 边界对比

| 输入 | 条件 | 错误 |
| --- | --- | --- |
| 非法字段、未知 ref 类型、前导零 | 不符合 Markdown 任何合法 ref | `REF_INVALID` |
| 合法 canonical heading ref | 当前解析结果无完全匹配项 | `REF_NOT_FOUND` |
| `doc:full` | 始终合法 | 进入全文读取路径 |

`REF_AMBIGUOUS` 在 Markdown adapter 当前解析结果中不会产生，因为 canonical ref 的唯一性排除了歧义场景。该错误仍然保留为共享稳定错误，供其它 adapter 在有歧义场景时使用。

## Display 职责与截断

### 职责分离

| 字段 | 职责 | 约束 |
| --- | --- | --- |
| `ref` | adapter 私有定位，供 read 原样消费 | 始终完整，不受 display 截断影响 |
| `display` | operation 专属可读语义 | 可以截断，但必须保留该 operation 所需非空核心语义 |

### Outline Display

Outline display 承载 heading title 或 breadcrumb 导航语义，并可附带 heading level、section cost 等现有摘要。ref 不包含 title 或 breadcrumb。

### Find Display

Find display 保留匹配位置附近的非空文本片段，并可补充对应 heading 的 title 或 breadcrumb。find 不得为了补充 heading 导航语义而删除命中上下文。

### 截断规则

超长 display 可按字符预算截断。发生截断时：

- 必须保留该 operation 所需的非空核心语义。
- 必须包含显式截断标记。
- ref 保持完整且不受影响。
- display 不得成为 read 解析 ref 或定位 heading 的输入。

## 默认值

以下默认值属于 `docnav-markdown` 配置域，可由该 CLI 的项目级或用户级配置覆盖：

| 参数 | 默认值 | 适用 operation |
| --- | --- | --- |
| `limit_chars` | `6000` | outline、read、find |
| `max_heading_level` | `3` | outline、find |

invoke 请求必须显式携带最终有限参数。分页操作的 page 省略时固定从 `1` 开始。

## 保证范围

Markdown adapter 保证：

- 同一解析结果中，outline 和 find 对同一 heading 生成相同 ref。
- canonical heading ref 在同一次解析结果中唯一。
- outline display 包含非空 heading title 或 breadcrumb 导航文本。
- find display 包含非空匹配位置附近文本片段。
- `doc:full` 始终可读取整篇文档。

Markdown adapter **不**保证：

- ref 跨文档修改或 parser 版本变化后继续指向同一 heading。

## 测试边界

Markdown adapter 测试必须覆盖本页拥有的行为语义：

- heading 识别、section 范围、frontmatter 和代码围栏排除。
- outline/find/read 的 ref 生成、原样读取、display 职责和截断边界。
- 默认 `limit_chars`、`max_heading_level` 和 page 从 `1` 开始的分页规则。
- Unicode 字符预算、超长 display 截断、ref 完整保留和分页前进。
- 重复 heading、重复路径、`doc:full`、`REF_INVALID` 和 `REF_NOT_FOUND` 边界。

测试层级、smoke case 组织和跨入口覆盖目标见 [测试策略](../testing.md) 与 [覆盖矩阵](../testing/coverage.md)。

## 验证入口

Markdown adapter 的开发期快捷命令：

```bash
pnpm run smoke:docnav-markdown:dev
pnpm --silent dnm outline <path>
pnpm --silent dnm read <path> --ref "<ref>"
pnpm --silent dnm find <path> --query "<text>"
pnpm --silent dnm outline <path> --output readable-json
```

省略 `--output` 时使用 `readable-view`；需要结构化阅读结果时显式使用 `readable-json`，需要完整协议 envelope 时使用 `protocol-json`。

交付前综合验证入口见 [测试策略](../testing.md)。
