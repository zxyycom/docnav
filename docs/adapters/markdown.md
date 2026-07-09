# Markdown Adapter

本文是 `docnav-markdown` 当前实现的导航行为和私有契约主文档。它覆盖 Markdown adapter 的 outline、read、find、ref grammar、结构快照语义、`doc:full`、错误分类、保证范围和验证入口，是 Markdown 实现与审计的长期规范来源。

共享协议和 `docnav` core 按共享 ref 契约原样传递 ref；Markdown heading ref 的解析归 Markdown adapter 拥有。共享 ref 最小契约见 [Ref](../ref-contract.md)。

## Heading 识别与 Section 范围

Markdown adapter 使用成熟 parser 识别有效 heading（H1–H6）。section 范围从目标 heading 开始，包含更深层 heading，在下一个同级或更高级 heading 前结束。

Outline heading 识别范围排除：

- frontmatter。
- 代码围栏内的伪 heading。

## Outline

本节定义 Markdown adapter 正常结构化 outline handler 的行为。若 navigation 标准 `outline_mode` 已解析为 `unstructured_full`，core-mediated navigation 在调用 linked Markdown adapter 的正常 outline handler 前直接返回整篇 Markdown 原文，`content_type` 为 `text/markdown`，且结果不包含 heading entries、`doc:full`、ref、page 或 continuation。`outline_mode` 不是 Markdown adapter-owned native option。

Markdown outline 按文档顺序返回扁平 heading entries。每条 entry 包含：

| 字段 | 内容 |
| --- | --- |
| `ref` | `H:L{line}:H{level}`，承载 line 和 level 结构坐标 |
| `label` | heading title |
| `kind` | `heading` |
| `location.line_start` | heading 的 1-based 起始行号 |
| `cost.measurements[]` | section 的 `lines`、`bytes`、`tokens` 成本，按该顺序报告，scope 为 `entry` |
| `metadata.heading_level` | Markdown heading level |

层级关系通过 `metadata.heading_level` 和 heading 的文档顺序表达。阅读输出可以把这些事实派生为包含 H 级别和成本摘要的 `display`。

### Document Head

Markdown adapter 将文档开头到第一个有效 Markdown heading 起点之前的原文区域定义为 document head。该区域保留原文字节序列对应的 Markdown 文本，包括 YAML frontmatter delimiter、空行、注释、普通前导正文和其它前导 Markdown；adapter 不解析 frontmatter 字段语义，也不把这些内容拆成独立 metadata。

当 document head 包含非空、非纯空白内容，且当前 structured outline 至少有一个可见 heading entry 时，outline 必须在 heading entries 前返回单条 document head entry。该 entry 的 ref 固定为 `HEAD:leading`。满足 eligibility 时该 entry 始终暴露，不需要调用方显式启用。

Document head entry 的 raw protocol facts 为：

| 字段 | 内容 |
| --- | --- |
| `ref` | `HEAD:leading`，adapter-owned document head ref |
| `label` | 非空 label，例如 `Document head` |
| `kind` | `document_head`，不得使用 `heading` |
| `location.line_start` | `1` |
| `cost.measurements[]` | document head 原文区域的 `lines`、`bytes`、`tokens` 成本，按该顺序报告，scope 为 `entry` |
| `metadata.document_region` | `leading` |

Outline result 不新增顶层 `frontmatter`、`metadata` 或 `document_head` 字段。Raw protocol 不包含 readable-only `display`；阅读输出可以从 `label`、`kind`、`location` 和成本 facts 派生可读 display。

### 可见性与 `max_heading_level`

Markdown adapter 的 registry-facing definition/factory 声明 `max_heading_level` adapter-owned typed-field declaration 和内置默认值 `3`，暴露 `--max-heading-level` CLI native option source，并通过 config-source projection 使用 `options.docnav-markdown.max_heading_level` 持久 config path。Adapter selection 后，`docnav-navigation` 将 selected Markdown definition 中的 `AdapterOptionSpec` 注册进 operation field set，按来源优先级解析该 option，并通过 typed-field helper 校验/提取为 `1..6` 的整数。Request construction/dispatch 边界再把该 typed value 作为 `NativeOptionHandoff` 交给 Markdown outline/find handler；handler 消费 typed handoff，不重新读取 raw CLI argv、raw config JSON 或未校验 source value。可见性过滤决定 outline 返回哪些 heading entries。可见 heading 的 line/level 结构坐标保持稳定，保证同一解析结果中的同一 heading 在 outline 和 find 中使用相同 ref。

### 无可见 heading 时的全文 ref

当前 outline 参数过滤后没有可见 heading，且 navigation `outline_mode` 为默认 `structured` 时，outline 返回单条 `doc:full` entry。该 ref 读取整篇 Markdown 文档。此 fallback 优先于 document head entry；outline 不在无可见 heading 时只返回 `HEAD:leading`。

## Find

Markdown find 搜索全文，但 match ref 指向当前 outline 参数下包含命中位置的可见区域。若多个可见 heading section 包含同一命中，选择源码位置最接近命中的可见 heading entry；命中位于 document head 内，且当前 structured outline 至少有一个可见 heading entry 时，find 使用 `HEAD:leading`。当前 outline 参数没有任何可见 heading、outline 使用 `doc:full` fallback 时，find 保持可继续 read 的 fallback 行为，必要时使用 `doc:full`。

每个 find match 返回 `label`、`kind: "match"` 和 `location.line_start`。`label` 保留匹配位置附近的非空文本片段；ref 由独立字段完整承载 line 和 level 结构坐标。阅读输出可以把 line 与 label 派生为紧凑 `display`。

## Read

### Heading Ref 读取

Markdown `read` 解析 `H:L{line}:H{level}` canonical heading ref，并在当前文档解析结果中精确匹配 line 和 level 全部相同的 heading。匹配成功时返回该 heading 的当前 Markdown section，`content_type` 为 `text/markdown`。

匹配条件严格限定为 line 和 level 两个结构字段。

Read result 的 `cost.measurements[]` 使用当前 ref 选中 Markdown text 的 `lines`、`bytes`、`tokens` 成本，按该顺序报告，scope 为 `selection`。分页后的 `content` 只返回当前 page，成本仍描述当前 ref 选中的 selection；`limit` 继续使用 Unicode 字符预算，不使用 token cost 作为分页预算。

### `doc:full` 读取

`read` 接受 `doc:full` 并返回整篇 Markdown 文档。`doc:full` 是 Markdown adapter 私有全文 ref，与 heading ref grammar 分列。

### Document Head Ref 读取

`read` 接受 `HEAD:leading` 并返回当前 Markdown document head 原文区域，`content_type` 为 `text/markdown`。如果该区域包含 YAML frontmatter delimiter，content 保留起止 delimiter。`limit` 和 `page` 行为沿用普通 read content 分页规则：分页预算按 Unicode 字符计数，分页后的 `content` 只返回当前 page，成本仍描述 `HEAD:leading` 选中的完整 document head selection。

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

`doc:full` 和 `HEAD:leading` 是 Markdown adapter-owned sentinel refs，与 heading ref grammar 分列。它们不改变 canonical heading ref 格式，也不允许调用方把 `HEAD:*` 解释为 heading 坐标。

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
| 非法字段、未知 ref 类型、前导零 | 位于 Markdown 合法 ref grammar 和 adapter-owned sentinel refs 之外 | `REF_INVALID` |
| 合法 canonical heading ref | 当前解析结果无完全匹配项 | `REF_NOT_FOUND` |
| `doc:full` | 始终合法 | 进入全文读取路径 |
| `HEAD:leading` | 当前解析结果存在 document head selection | 进入 document head 读取路径 |

`doc:full` 是 Markdown adapter-owned navigation ref，不是共享层 fallback 或入口默认 ref。共享层只校验 explicit ref 是非空字符串；`doc:full` 的接受、读取粒度和返回 payload shape 由 Markdown adapter read 语义负责。

Markdown adapter 当前解析结果中的 canonical ref 唯一，因此该 adapter 的 read 路径使用确定性定位。`REF_AMBIGUOUS` 仍是共享稳定错误，供其它 adapter 在有歧义场景时使用。

## Item Facts 职责与截断

### 职责分离

| 字段 | 职责 | 约束 |
| --- | --- | --- |
| `ref` | adapter 私有定位，供 read 原样消费 | 始终完整；其它 facts 截断独立处理 |
| `label` | operation 专属最小语义 | 非空；可压缩但必须保留定位或匹配核心 |
| `kind`、`location`、`summary`、`excerpt`、`cost`、`metadata` | 可选结构化事实 | 能稳定表达时返回；预算不足时可省略或压缩 |

### Outline Facts

Outline heading entry 的 `label` 承载 heading title；`metadata.heading_level` 承载 heading level；`cost.measurements[]` 承载 section 的 `lines`、`bytes`、`tokens` 成本。ref 承载 line 和 level 结构坐标。

Document head outline entry 的 `label` 必须非空，`kind` 必须为非 heading kind `document_head`，`location.line_start` 固定为 `1`，`metadata.document_region` 固定为 `leading`。这些是 raw protocol facts；readable `display` 只能由输出层从这些 facts 派生，不能写回 raw protocol contract。

### Find Facts

Find entry 的 `label` 保留匹配位置附近的非空文本片段，`location.line_start` 保留命中所在行。命中上下文和 heading ref 共同构成 find 的可读内容来源。

### 截断规则

Markdown adapter 将通用 `limit` 解释为 Unicode 字符预算。超长 item facts 可按字符预算截断或省略。发生截断时：

- 必须保留该 operation 所需的非空核心语义。
- 字符预算允许时使用 `...` 作为显式截断标记。
- 当完整 ref 已耗尽或几乎耗尽预算时，`label` 可压缩为最小非空文本，并可省略 `summary`、`excerpt`、`cost` 或 `metadata` 等补充事实。
- ref 始终完整。
- read 的定位输入始终是 `ref` 字段。

## 默认值

以下默认值属于 core-linked `docnav-markdown` adapter definition 和 adapter-owned typed-field declarations；来源解析、native option handoff、selected adapter declaration registration 和 request construction 策略见 [Navigation Input Resolution](../navigation-input-resolution.md)。Markdown 本页记录格式私有默认值、operation 适用范围和 adapter-owned option semantics：

| 参数 | 默认值 | 适用 operation |
| --- | --- | --- |
| `pagination.enabled` | `true` | outline、read、find |
| `limit` | `6000` | outline、read、find |
| `output` | `readable-view` | outline、read、find、info |
| `max_heading_level` | `3` | outline、find |

Protocol request `arguments` 表达当前 document operation 进入 adapter 时的显式输入；省略参数已在 request construction 前由 `docnav-navigation` 从 explicit、project config、user config 或 built-in default 中解析补足。分页操作的 page 省略时固定从 `1` 开始。

Core `docnav` 配置源可以携带以下与 Markdown adapter 相关的 source values；项目配置路径为 `<project-root>/.docnav/docnav.json`，用户配置由 core 用户配置文件提供，最终解析由 `docnav-navigation` 执行：

| 配置字段 | 参数来源 | Markdown 语义 |
| --- | --- | --- |
| `defaults.pagination.enabled` | 通用 `pagination.enabled` | outline、read 和 find 的默认分页状态 |
| `defaults.pagination.limit` | 通用 `limit` | outline、read 和 find 的默认 Unicode 字符预算 |
| `defaults.output` | 通用 `output` | document operation 默认输出模式 |
| `outline.mode_rules[]` | navigation-owned selector | 可按 Markdown document path 选择 `structured` 或 `unstructured_full` |
| `outline.auto_full_read.thresholds[]` | navigation-owned selector | 可按 selected adapter id `docnav-markdown` 和 definition-declared full-read cost unit 选择 `unstructured_full` |
| `options.docnav-markdown.max_heading_level` | Markdown native option | outline 和 find 的可见 heading 粒度 |

Markdown native option 和 outline selector 的 core 配置参考形状见 [docnav-markdown-config.schema.json](../schemas/docnav-markdown-config.schema.json)。基础 native option 示例见 [docnav-markdown-config.json](../examples/json/docnav-markdown-config.json)，selector 示例见 [示例索引](../examples/README.md#配置示例)。这些文件只作为配置填写提示和示例校验；配置发现、字段映射、来源合并、失败处理和 runtime 参数校验由 [Navigation Input Resolution](../navigation-input-resolution.md) 与 [适配器契约](../adapter-contract.md#文档操作执行边界) 定义，runtime 不要求先加载该 schema。Config source 只提供 raw source value，`max_heading_level` 的 `1..6` 语义由 Markdown typed-field declaration 表达，并在 request construction 前校验/提取。

`options.docnav-markdown.max_heading_level` 是 Markdown adapter-owned native option config source path；`docnav-markdown` 是当前 registry adapter id。CLI/config source 通过 Markdown definition 中的 `AdapterOptionSpec` declaration 进入 navigation input resolution；adapter selection 后，`docnav-navigation` 只解析 selected Markdown definition 注册出的 options。Type mismatch 或 `1..6` 范围外值在 typed-field validation/extraction 阶段返回带来源的 diagnostic。旧裸 `options.max_heading_level` 不兼容、不迁移，只按普通 unknown/invalid config path 处理。

Markdown definition 同时声明 full-read capability group：content hook 返回整篇 Markdown 原文，cost measurement units 为 `lines`、`bytes` 和 `tokens`，result facts 由 content hook 返回的 cost facts 提供。Navigation 的 path rule 或 cost threshold 允许 `unstructured_full` 时，从 selected Markdown definition 读取该 group；Markdown adapter 仍保留正常 `outline`、`read`、`find` 和 `info` handler 作为必需 operation handlers。

## 保证范围

Markdown adapter 保证：

- 同一解析结果中，outline 和 find 对同一 heading 生成相同 ref。
- canonical heading ref 在同一次解析结果中唯一。
- outline entry 包含非空 heading title `label`、heading `kind`、line location、heading level metadata 和 section cost。
- 满足 eligibility 时，outline 在 heading entries 前始终暴露 `HEAD:leading` document head entry；空或纯空白 document head 不暴露该 entry。
- 无可见 heading 时，structured outline 保留单条 `doc:full` fallback。
- find entry 包含非空匹配位置附近文本片段 `label`、match `kind` 和 line location。
- document head 内的 find match 在有可见 heading 时返回 `HEAD:leading`；fallback 场景返回仍可 read 到命中文本的 ref。
- `doc:full` 始终可读取整篇文档；`HEAD:leading` 可读取 document head 原文并返回 `text/markdown`。

Markdown adapter 的结构快照边界：

- ref 跨文档修改或 parser 版本变化后的匹配结果由当前解析结果决定。

## 测试边界

Markdown adapter 测试必须覆盖本页拥有的行为语义：

- heading 识别、section 范围、frontmatter 和代码围栏排除。
- document head 范围、满足 eligibility 时始终暴露 `HEAD:leading`、空或纯空白 document head 不暴露 entry、无可见 heading 时 `doc:full` fallback。
- document head read 保留 frontmatter delimiter 和普通前导正文原文，`content_type` 为 `text/markdown`，并按 Unicode 字符预算分页。
- find 命中 document head 到 `HEAD:leading` read 的 roundtrip，以及 fallback 场景仍可 read 到命中文本。
- outline/find/read 的 ref 生成、原样读取、item facts 职责、成本 measurement 和截断边界。
- 静态 descriptor 暴露的默认 `pagination.enabled`、`limit`、`max_heading_level` 和 page 从 `1` 开始的分页规则。
- Core CLI config source descriptor/path handoff、path context、navigation-owned default config absence、invalid config failure、`defaults.output`、navigation input resolution native option handoff 和 Markdown adapter 对 `options.docnav-markdown.max_heading_level` 的 outline/find 可观察行为。
- Unicode 字符预算、超长 item facts 截断、ref 完整保留和分页前进。
- 重复 heading、重复路径、`doc:full`、`REF_INVALID` 和 `REF_NOT_FOUND` 边界。

测试层级、smoke case 组织和跨入口覆盖目标见 [测试策略](../testing.md) 与 [覆盖矩阵](../testing/coverage.md)。

## 验证入口

Markdown adapter 的开发期快捷命令通过 core CLI 验证：

```bash
bun --silent run dnm outline <path>
bun --silent run dnm read <path> --ref "<ref>"
bun --silent run dnm find <path> --query "<text>"
bun --silent run dnm outline <path> --output readable-json
```

省略 `--output` 时使用 `readable-view`；需要结构化阅读结果时显式使用 `readable-json`，需要完整协议 envelope 时使用 `protocol-json`。

交付前综合验证入口见 [测试策略](../testing.md)。
