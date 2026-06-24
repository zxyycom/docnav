# Markdown Adapter

本文是 `docnav-markdown` 当前实现的导航行为和私有契约主文档。它覆盖 Markdown adapter 的 outline、read、find、ref grammar、结构快照语义、`doc:full`、错误分类、保证范围和验证入口，是 Markdown 实现与审计的长期规范来源。

共享协议和 `docnav` core 按共享 ref 契约原样传递 ref；Markdown heading ref 的解析归 Markdown adapter 拥有。共享 ref 最小契约见 [Ref](../ref-contract.md)。

## Heading 识别与 Section 范围

Markdown adapter 使用成熟 parser 识别有效 heading（H1–H6）。section 范围从目标 heading 开始，包含更深层 heading，在下一个同级或更高级 heading 前结束。

Outline heading 识别范围排除：

- frontmatter。
- 代码围栏内的伪 heading。

## Outline

Markdown outline 按文档顺序返回扁平 heading entries。每条 entry 包含：

| 字段 | 内容 |
| --- | --- |
| `ref` | `H:L{line}:H{level}`，承载 line 和 level 结构坐标 |
| `display` | heading title 或 breadcrumb 导航文本，可含 heading level、section cost 等紧凑摘要 |

层级关系通过 display 中的 heading title 或 breadcrumb 表达。

### 可见性与 `max_heading_level`

Markdown adapter 内置 `max_heading_level: 3`（可由 `docnav-markdown` 配置字段覆盖）。可见性过滤决定 outline 返回哪些 heading entries；可见 heading 的 line/level 结构坐标保持稳定，保证同一解析结果中的同一 heading 在 outline 和 find 中使用相同 ref。

### 无可见 heading 时的全文 ref

当前 outline 参数过滤后没有可见 heading 时，outline 返回单条 `doc:full` entry。该 ref 读取整篇 Markdown 文档。

## Find

Markdown find 搜索全文，但 match ref 指向当前 outline 参数下离命中位置最近的 heading entry。最近 heading 按源码位置判断：命中位于两个 outline entry 之间时选择距离更近的一项，距离相同则选择前一项。若当前 outline 参数没有任何 entry，find 使用 `doc:full`。

每个 find match 的 display 保留匹配位置附近的非空文本片段，并可补充对应 heading 的 title 或 breadcrumb。ref 由独立字段完整承载 line 和 level 结构坐标。

## Read

### Heading Ref 读取

Markdown `read` 解析 `H:L{line}:H{level}` canonical heading ref，并在当前文档解析结果中精确匹配 line 和 level 全部相同的 heading。匹配成功时返回该 heading 的当前 Markdown section，`content_type` 为 `text/markdown`。

匹配条件严格限定为 line 和 level 两个结构字段。

### `doc:full` 读取

`read` 接受 `doc:full` 并返回整篇 Markdown 文档。`doc:full` 是 Markdown adapter 私有全文 ref，与 heading ref grammar 分列。

## Heading Ref Grammar

Markdown heading ref 使用以下 canonical 格式：

```text
H:L{line}:H{level}
```

| 字段 | 含义 | 约束 |
| --- | --- | --- |
| `H` | heading ref 类型标识 | 固定前缀 |
| `L{line}` | heading 的 1-based 起始行号 | 首位为 `1`–`9` 的十进制正整数 |
| `H{level}` | Markdown heading level | `1`–`6` |

Canonical grammar 正则：`^H:L([1-9][0-9]*):H([1-6])$`。

### 长度保证

ref 长度由 line 的十进制位数决定。字段前缀（`L`、`H`）提供基础语义提示，支持日志和人工审计。

### 同一解析结果中的唯一性

当前 Markdown parser 的同一解析结果中，每个有效 heading 拥有唯一 1-based 起始行号，因此 line + level canonical ref 在同一次解析结果中唯一。该唯一性是 Markdown adapter 的私有选择；其它 adapter 由各自 ref grammar 决定定位语义。

### 重复 Heading

位于不同行的重复 title 或重复 breadcrumb heading 根据自身 line 和 level 获得不同 ref。read 按 line 和 level 定位 heading。

## 结构快照语义

Markdown heading ref 是生成时解析结果中的结构坐标；当前解析结果中的 line 和 level 是 heading 身份输入。

文档内容或 parser 结果变化后，同一个格式合法的 ref 可能：

- 找不到匹配 heading，返回 `REF_NOT_FOUND`；
- 匹配当前结构中坐标相同的另一个 heading；
- 在结构坐标未变化时继续匹配当前 heading。

以上结果均符合本 adapter 的契约。调用方在需要当前结构时重新执行 outline 或 find 获取当前 ref；过期 ref 的结果由当前解析结果决定。

## 错误分类

Markdown adapter 在 `read` 中按以下边界映射 ref 错误：

### `REF_INVALID`

输入 ref 是非空字符串且落在 Markdown adapter 当前合法 ref grammar 之外时，返回 `REF_INVALID`。

- 稳定 details 包含原始 `ref` 和非空 `reason`。

### `REF_NOT_FOUND`

输入 ref 符合 canonical heading grammar，但当前解析结果中没有 line 和 level 全部匹配的 heading 时，返回 `REF_NOT_FOUND`。

### 边界对比

| 输入 | 条件 | 错误 |
| --- | --- | --- |
| 非法字段、未知 ref 类型、前导零 | 位于 Markdown 合法 ref grammar 之外 | `REF_INVALID` |
| 合法 canonical heading ref | 当前解析结果无完全匹配项 | `REF_NOT_FOUND` |
| `doc:full` | 始终合法 | 进入全文读取路径 |

Markdown adapter 当前解析结果中的 canonical ref 唯一，因此该 adapter 的 read 路径使用确定性定位。`REF_AMBIGUOUS` 仍是共享稳定错误，供其它 adapter 在有歧义场景时使用。

## Display 职责与截断

### 职责分离

| 字段 | 职责 | 约束 |
| --- | --- | --- |
| `ref` | adapter 私有定位，供 read 原样消费 | 始终完整；display 截断独立处理 |
| `display` | operation 专属可读语义 | 可以截断，但必须保留该 operation 所需非空核心语义 |

### Outline Display

Outline display 承载 heading title 或 breadcrumb 导航语义，并可附带 heading level、section cost 等现有摘要。ref 承载 line 和 level 结构坐标。

### Find Display

Find display 保留匹配位置附近的非空文本片段，并可补充对应 heading 的 title 或 breadcrumb。命中上下文和 heading 导航语义共同构成 find display 的可读内容。

### 截断规则

超长 display 可按字符预算截断。发生截断时：

- 必须保留该 operation 所需的非空核心语义。
- 字符预算允许时使用 `...` 作为显式截断标记。
- 当完整 ref 已耗尽或几乎耗尽预算时，display 可压缩为最小非空文本。
- ref 始终完整。
- read 的定位输入始终是 `ref` 字段。

## 默认值

以下默认值属于 `docnav-markdown` 标准参数 registration；配置字段映射、来源合并和 `invoke` 入口策略由 [标准参数](../standard-parameters.md) 定义。Markdown 本页只记录格式私有默认值及其 operation 适用范围：

| 参数 | 默认值 | 适用 operation |
| --- | --- | --- |
| `limit_chars` | `6000` | outline、read、find |
| `output` | `readable-view` | outline、read、find、info |
| `max_heading_level` | `3` | outline、find |

Adapter `invoke` request `arguments` 是显式输入；省略的已注册参数可由 `invoke` 入口的配置或默认值补足。分页操作的 page 省略时固定从 `1` 开始。

`docnav-markdown` JSON 配置文件首期支持以下字段：

| 配置字段 | 参数来源 | Markdown 语义 |
| --- | --- | --- |
| `defaults.limit_chars` | 通用 `limit_chars` | outline、read 和 find 的默认字符预算 |
| `defaults.output` | 通用 `output` | document operation 默认输出模式 |
| `options.max_heading_level` | Markdown native option | outline 和 find 的可见 heading 粒度 |

配置文件形状见 [docnav-markdown-config.schema.json](../schemas/docnav-markdown-config.schema.json) 和 [docnav-markdown-config.json](../examples/json/docnav-markdown-config.json)。这些文件只作为配置填写提示、示例校验和 adapter package 打包参考；配置发现、字段映射、来源合并、失败处理和 runtime 参数校验由 [标准参数](../standard-parameters.md) 与 [适配器契约](../adapter-contract.md#标准参数消费边界) 定义，runtime 不要求先加载该 schema。

## 保证范围

Markdown adapter 保证：

- 同一解析结果中，outline 和 find 对同一 heading 生成相同 ref。
- canonical heading ref 在同一次解析结果中唯一。
- outline display 包含非空 heading title 或 breadcrumb 导航文本。
- find display 包含非空匹配位置附近文本片段。
- `doc:full` 始终可读取整篇文档。

Markdown adapter 的结构快照边界：

- ref 跨文档修改或 parser 版本变化后的匹配结果由当前解析结果决定。

## 测试边界

Markdown adapter 测试必须覆盖本页拥有的行为语义：

- heading 识别、section 范围、frontmatter 和代码围栏排除。
- outline/find/read 的 ref 生成、原样读取、display 职责和截断边界。
- 默认 `limit_chars`、`max_heading_level` 和 page 从 `1` 开始的分页规则。
- `docnav-markdown` direct CLI 配置优先级、路径覆盖、配置源跳过 warning、`defaults.output` 和 `options.max_heading_level` 对 outline/find 的可观察行为。
- Unicode 字符预算、超长 display 截断、ref 完整保留和分页前进。
- 重复 heading、重复路径、`doc:full`、`REF_INVALID` 和 `REF_NOT_FOUND` 边界。

测试层级、smoke case 组织和跨入口覆盖目标见 [测试策略](../testing.md) 与 [覆盖矩阵](../testing/coverage.md)。

## 验证入口

Markdown adapter 的开发期快捷命令：

```bash
bun run smoke:docnav-markdown:dev
bun --silent run dnm outline <path>
bun --silent run dnm read <path> --ref "<ref>"
bun --silent run dnm find <path> --query "<text>"
bun --silent run dnm outline <path> --output readable-json
```

省略 `--output` 时使用 `readable-view`；需要结构化阅读结果时显式使用 `readable-json`，需要完整协议 envelope 时使用 `protocol-json`。

交付前综合验证入口见 [测试策略](../testing.md)。
