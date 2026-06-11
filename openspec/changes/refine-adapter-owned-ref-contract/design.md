本 change 的目标是明确 adapter-owned ref 的共享契约边界，并用有界、可审计的结构坐标改进 Markdown heading ref。当前设计已通过审计并解除实施门禁，尚未应用到现行主规范或实现。

## Context

ref 由格式 adapter 生成和解释。共享协议、`docnav` core 和 MCP 只需要承载非空字符串并原样传递，不需要理解 ref 表示一个位置、范围、查询、句柄还是其它 adapter 私有概念。

当前 Markdown heading ref 使用 `L{line}:{path}` 或 `L{line}#{ordinal}:{path}`。完整 breadcrumb 让 ref 长度受标题文本控制。Markdown parser 已经为每个有效 heading 保存 1-based line、heading level 和全文 index，因此可以直接生成不含标题内容的结构 ref。

本设计同时纠正共享 ref 契约的抽象层级：共享层定义载体和所有权，不替 adapter 规定唯一性、稳定性、消歧或读取保证；Markdown adapter 再按自身需求选择更具体的行为。

## Goals / Non-Goals

**Goals:**

- Markdown heading ref 不包含 title、breadcrumb、内容摘要或字符集相关文本。
- ref 长度在常见文档中保持较短，并且只随 line 和 heading 数量的十进制位数缓慢增长。
- 字段前缀提供基础语义提示，使日志、CLI 输出和人工审计无需记忆纯数字字段顺序。
- Markdown outline 和 find 对同一解析结果中的同一 heading 生成相同 ref。
- 共享 ref 契约强制保留 `outline/find -> ref -> read` 流程、adapter 所有权、非空字符串载体、原样传递和稳定错误通道。
- 非法 ref grammar 与合法但无匹配的 ref 使用不同稳定错误表达。
- Markdown 私有导航行为由独立 adapter 文档拥有。

**Non-Goals:**

- 为所有 adapter 规定唯一定位、消歧、身份稳定性、读取成功或一对一映射。
- 为 Markdown ref 提供文档版本、mtime、内容 hash、过期检测或跨修改身份保证。
- 在 core 或 MCP 中解析、转换或兼容 Markdown ref。
- 为旧 Markdown heading ref 建立长期兼容分支或专属错误语义。

## Confirmed Decisions and Review Boundary

以下决策已经确认，属于本 change 的输入约束，不是待解决的开放问题：

1. `outline/find -> ref -> read` 是强制且稳定的共享调用流程。
2. core、协议和 MCP 只校验共享字段 shape，并将非空 opaque ref 原样传给按 path 选定的 adapter。
3. 共享层不保证 `read` 必然成功、唯一定位或返回特定区域。ref 的 grammar、适用 operation、有效条件、唯一性、消歧、读取结果和错误分类由 adapter 专属契约定义。
4. 该边界是正确性责任的分层，不是为了缩短 ref 而放弃正确性：共享层负责 adapter 选择、原样传递和稳定错误映射；adapter 负责其生成、解释、定位和错误行为符合自身契约。
5. Markdown heading ref 是结构快照，不是内容身份。文档变化后旧 ref 可能失效、继续匹配或匹配其它当前 heading，这是 Markdown adapter 明确接受的 trade-off。
6. Markdown outline display 承载标题或 breadcrumb；find display 保留匹配位置附近的文本片段，并可补充 heading 导航语义。超长 display 可以按字符预算截断。
7. 新 grammar 生效后只生成和接受新 heading ref；旧 heading ref 统一返回 `REF_INVALID`，不提供兼容读取或双 grammar 迁移。

后续审查和实现应验证 artifacts、实现、测试和错误映射是否准确落实这些决策。不同设计偏好、通用 API 惯例、旧 ref 兼容偏好、内容身份偏好或要求共享层提供更强定位保证，不构成重新讨论这些决策的理由。

只有出现以下情况时才重新评估对应决策：与现行主规范存在可定位的实质冲突；存在明确不可实现条件；出现可复现的契约缺陷；或用户明确要求修改该决策。

## Decisions

### 1. 共享 ref 契约只定义载体、所有权和传递

共享层强制规定稳定调用流程：adapter 在 outline 或 find 中生成 ref；调用方将 path 和 ref 原样提交给 read；core 根据 path 选择 adapter 并原样传递 ref；adapter 自行解释 ref，并返回读取结果或稳定错误。

共享层将 ref 定义为 adapter 生成和解释的非空 opaque string。`docnav`、MCP、schema 和其它接入层只校验 ref 是非空字符串，并把收到的值原样传给选定 adapter。

“可传给 read”只表示 ref 可以作为 read 请求字段跨共享层传输，不表示共享层保证 adapter 接受该值、完整消费该值、唯一定位、成功返回或在未来文档状态下保持含义。以下语义全部由 adapter 自行定义并在其规范中记录：

- ref grammar 和内部字段；
- 一个 ref 对应零个、一个或多个区域；
- 多个 ref 是否可以指向同一区域；
- 是否执行唯一性校验或消歧；
- ref 是否适用于 outline、find、read 或其它 operation；
- 文档或 parser 变化后的行为；
- 非法、未匹配、歧义或其它失败如何映射到稳定错误。

该边界让后续 JSON、YAML、TOML、INI 或其它 adapter 可以选择符合自身结构的 ref，而不继承 Markdown 的 heading 定位模型。共享调用链保持稳定，但 adapter 的解释结果保持格式私有。

正确性责任按所有权分层：共享层的正确性是选择正确 adapter、保持 ref 不变并一致映射稳定错误；adapter 的正确性是其生成、解释、定位和失败行为符合自身公开契约。共享层不了解 adapter grammar、文档状态和定位模型，因此既无法也不应替 adapter 保证 ref 一定被 `read` 接受或成功读取。

### 2. Markdown heading ref 使用 `H:L{line}:H{level}:I{index}`

- 首个 `H` 表示 heading ref 类型。
- `L{line}` 表示 heading 的 1-based 起始行号。
- `H{level}` 表示 Markdown heading level，范围为 `1` 到 `6`。
- `I{index}` 表示 heading 在全文有效 headings 中的 1-based 顺序号。

三个数字字段使用不带前导零的十进制表示。canonical grammar 为 `^H:L([1-9][0-9]*):H([1-6]):I([1-9][0-9]*)$`。`index` 在 `max_heading_level` 等可见性过滤前确定，因此同一次解析结果中的同一 heading 不因 outline 或 find 的可见性参数重新编号。

这里的“短”不是要求每个具体标题下都比旧 ref 字符数更少，而是保证 ref 长度不再受标题长度、breadcrumb 深度和 Unicode 文本影响。在多数文档中，固定前缀与少量十进制数字提供可预测的有界成本。

字段前缀是有意保留的审计信息。相比 `1:2:3` 等纯数字串，`L`、`H`、`I` 能让日志和人工检查直接识别字段含义，并为未来增加其它 Markdown ref 类型保留类型边界。

未采用的方案：

- breadcrumb 或 title：长度继续受文档文本控制。
- 内容摘要或 hash：增加算法、规范化、编码和碰撞契约，但本 change 不需要内容身份保证。
- 纯 index：更短，但失去行号和 heading level 的人工审计信息。
- 无标签数字元组：字符更少，但字段含义依赖顺序记忆，降低诊断可读性。
- line + level：在当前 parser 结果中通常足够区分 heading，但缺少独立全文顺序坐标，无法表达本设计选择的三字段一致性检查。

### 3. Markdown read 精确匹配当前解析结果中的三个字段

Markdown `read` 解析 canonical heading ref，并在当前解析结果中查找 line、level 和 index 全部相同的 heading。匹配成功时读取该 heading 的当前 section；没有匹配项时返回 `REF_NOT_FOUND`。

该精确匹配是 Markdown adapter 的私有选择，不提升为共享 ref 保证。当前 parser 为每个有效 heading 生成唯一 index，因此 canonical ref 在同一次解析结果中唯一；其它 adapter 无需采用相同保证。

### 4. Markdown 导航语义由 display 承载

heading ref 不再包含 title 或 breadcrumb 后，outline 的 `display` 必须提供对应的标题或 breadcrumb 导航语义，并可以附带 heading level、section cost 等现有摘要。find 的 `display` 必须保留匹配位置附近的文本片段，并可以补充对应 heading 的 title 或 breadcrumb。

display 受现有字符预算约束。outline 的超长标题或 breadcrumb、find 的超长匹配片段或补充导航文本都可以截断；截断后必须保留该 operation 所需的非空核心语义，发生省略时必须包含显式截断标记。截断不得修改、缩短或从 display 反推 ref。ref 始终由独立字段完整承载。

### 5. Markdown ref 是结构快照，不是内容身份

Markdown heading ref 只描述生成时解析结果中的结构坐标。文档内容或 parser 结果变化后，同一个 ref 可能：

- 不再匹配并返回 `REF_NOT_FOUND`；
- 匹配当前结构中的另一个 heading；
- 在结构坐标未变化时继续匹配当前 heading。

这些结果都符合本 adapter 的契约。调用方若需要读取当前结构，应重新执行 outline 或 find 获取当前 ref；契约不要求调用方检测文档是否变化，也不声称旧 ref 一定失败。

接受该 trade-off 的理由是：Docnav 的主链路面向即时、有限、可继续的导航，不把 ref 作为持久化内容身份。加入版本或内容身份会扩大 ref、协议和缓存语义，超出本 change 目标。

### 6. 非法 grammar 使用 `REF_INVALID`

稳定错误增加 `REF_INVALID`，details 为：

- `ref`：调用方提供的原始非空 ref；
- `reason`：adapter 提供的可机器识别或可诊断原因。

Markdown 按以下边界映射错误：

- 输入不符合当前 canonical heading grammar，且不是 adapter 定义的其它合法 ref：返回 `REF_INVALID`。
- 输入符合 canonical heading grammar，但当前解析结果中没有三个字段全部匹配的 heading：返回 `REF_NOT_FOUND`。
- `doc:full` 是合法的 Markdown 私有 ref，进入全文读取路径。

旧 `L{line}:{path}`、`L{line}#{ordinal}:{path}` 和显式 `L{line}#1:{path}` 只是不符合当前 grammar 的输入示例，不建立 `legacy ref` 类型、兼容分支或专属错误。

未复用 `INVALID_REQUEST`，因为 read request 的传输 shape 和 ref 字段类型仍然合法，失败发生在 adapter 解释其私有 ref grammar 时。`REF_INVALID` 能保持传输校验与 adapter 语义校验的边界清楚。

### 7. `doc:full` 继续属于 Markdown adapter

当当前 outline 参数下没有可见 heading 时，Markdown adapter 返回单条 `doc:full` entry。`read` 使用该 ref 返回整篇 Markdown 文档。

`doc:full` 只由 Markdown adapter 的规范和文档定义。共享 ref 契约不建立全文 ref 类型，也不要求其它 adapter 提供等价语义。

### 8. Markdown 行为使用独立主文档

新增 `docs/adapters/markdown.md`，作为当前 Markdown adapter 行为的长期主文档，覆盖：

- heading 识别、outline 可见性和 section 范围；
- heading ref 的生成、读取和结构快照保证；
- `doc:full` 的生成条件和读取行为；
- find match 的 ref 归属；
- `REF_INVALID` 与 `REF_NOT_FOUND` 的边界；
- Markdown 默认值和验证入口。

`docs/refs.md` 只保留共享最小契约并链接到 adapter 专页。`docs/references/markdown-navigator.md` 只记录外部来源和迁移依据，不拥有现行行为规则。

## Risks / Trade-offs

- [旧 Markdown ref 无法按新 grammar 读取] -> 这是显式 breaking change；调用方通过当前 outline 或 find 获取新 ref。旧格式只保留为非法 grammar 测试输入。
- [结构变化后旧 ref 可能读取不同 heading] -> 在 Markdown 专属文档中明确结构快照语义，不提供内容身份保证。
- [共享层只保证调用流程和原样传递] -> 每个 adapter 必须在自己的规范中说明实际 ref 行为；core 和 MCP 保持完全格式无关。
- [新增稳定错误扩大协议枚举] -> 同步更新错误规则、schema、示例、Rust/JS 映射和跨输出层测试。
- [ref 示例分散] -> 通过全仓受限搜索、文档验证和 workspace verification 同步更新。

## Migration Plan

1. 审核并确认共享 ref 最小契约、Markdown 三字段 grammar、结构快照语义和 `REF_INVALID` 边界。
2. 扩展稳定错误规则、protocol/readable schema 和输出层映射。
3. 更新 Markdown heading ref 的生成、解析、find 归属和错误映射。
4. 新增 `docs/adapters/markdown.md`，并从共享文档和参考文档迁移 Markdown 私有行为。
5. 更新规范、示例、fixture、golden output 和测试。
6. 运行局部验证和 `pnpm run verify:docnav-workspace`。

回退时整体恢复旧 Markdown heading ref 和原错误集合，不在 core 或 MCP 中增加格式兼容逻辑。

## Open Questions

- 无。阻塞级审计已经完成，artifacts 与 Confirmed Decisions and Review Boundary 一致，可以进入实现。只有该节列出的重新评估条件成立时，才重新开启对应决策。
