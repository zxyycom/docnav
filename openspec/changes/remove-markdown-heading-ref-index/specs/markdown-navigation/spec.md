本 spec delta 定义 Markdown heading ref 去掉 `I{index}` 后的规范行为；它只在 `openspec/changes/remove-markdown-heading-ref-index/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## MODIFIED Requirements

### Requirement: Markdown heading ref 必须使用带字段标识的 canonical 格式
`docnav-markdown` MUST 为 heading 生成 `H:L{line}:H{level}` 格式的 canonical ref。

- 首个 `H` MUST 标识 heading ref 类型。
- `L{line}` 中的 `line` MUST 是 heading 的 1-based 起始行号。
- `H{level}` 中的 `level` MUST 是 `1` 到 `6` 的 Markdown heading level。
- 两个数值字段 MUST 使用不带前导零的十进制表示。

canonical ref MUST 匹配 `^H:L([1-9][0-9]*):H([1-6])$`。ref 的生成 MUST 独立于 heading title、breadcrumb、字符集和全文 heading index。这里的长度保证 MUST 表达为不受标题文本、breadcrumb 深度和全文 heading 数量影响，只随 line 与 level 的十进制位数增长。

#### Scenario: 生成 canonical heading ref
- **WHEN** 第 1 个有效 heading 是位于第 1 行的 H1
- **AND** 第 2 个有效 heading 是位于第 5 行的 H2
- **THEN** outline 分别输出 `H:L1:H1` 和 `H:L5:H2`

#### Scenario: Heading ref 不包含过滤前 index
- **WHEN** 全文依次包含 H1、H4 和 H2
- **AND** 当前 `max_heading_level` 过滤掉 H4
- **THEN** H2 的 ref 使用 `H:L{line}:H2`
- **THEN** outline 和 find 对该 H2 返回相同 ref

#### Scenario: 文本内容不进入 heading ref
- **WHEN** heading 包含极长标题、深层 breadcrumb 或 Unicode 文本
- **THEN** ref 仍只由 heading 类型、`L{line}` 和 `H{level}` 构成

#### Scenario: 重复 heading 生成不同 ref
- **WHEN** 文档包含重复 title 或重复 breadcrumb 且这些 heading 位于不同行
- **THEN** 每个 heading 根据自身 line 和 level 获得不同 ref

### Requirement: Markdown outline 和 find 必须在 display 中保留各自的可读语义
`docnav-markdown` MUST 在 heading ref 不包含标题文本时，通过 outline 的 `display` 提供 heading title 或 breadcrumb 导航语义。outline display MAY 同时包含 heading level、section cost 或其它紧凑摘要。

find 的 `display` MUST 保留匹配位置附近的非空文本片段，并 MAY 补充对应 heading 的 title 或 breadcrumb。find 不得为了补充 heading 导航语义而删除命中上下文。

outline 的超长 title 或 breadcrumb，以及 find 的超长匹配片段或补充导航文本，MAY 按字符预算截断。截断后 MUST 保留该 operation 所需的非空核心语义；发生省略时 MUST 包含显式截断标记。截断 MUST 只影响 display，不得修改 ref；完整 ref MUST 始终由 `ref` 字段承载。display MUST NOT 成为 read 解析 ref 或定位 heading 的输入。

#### Scenario: Ref 与 display 分离职责
- **WHEN** outline 返回 heading entry
- **THEN** ref 使用 `H:L{line}:H{level}` 且不包含 title 或 breadcrumb
- **THEN** display 包含非空的 title 或 breadcrumb 文本片段

#### Scenario: Find display 保留命中上下文
- **WHEN** find 返回匹配 entry
- **THEN** display 包含匹配位置附近的非空文本片段
- **THEN** display 可以补充对应 heading 的 title 或 breadcrumb
- **THEN** ref 仍由独立字段完整承载

#### Scenario: 超长 display 可以截断
- **WHEN** outline 的 heading 导航文本或 find 的匹配片段超过当前字符预算允许的 display 长度
- **THEN** adapter 截断对应 display 文本、保留该 operation 所需的非空核心语义、输出显式截断标记并保持分页能够前进
- **THEN** ref 字段保持完整且不受截断影响

### Requirement: Markdown read 必须按当前解析结果精确匹配 canonical heading ref
`docnav-markdown read` MUST 解析 `H:L{line}:H{level}`，并在当前文档解析结果中匹配 line 和 level 全部相同的 heading。匹配成功时 MUST 返回该 heading 的当前 Markdown section；没有匹配项时 MUST 返回 `REF_NOT_FOUND`。

该精确匹配和当前解析结果中的唯一性 MUST 属于 Markdown adapter 私有行为，不得提升为所有 adapter 的共享 ref 保证。`read` MUST NOT 使用 heading title、breadcrumb、section 内容、全文 heading index 或其摘要补充匹配。

#### Scenario: 读取 canonical heading ref
- **WHEN** 调用方把当前 outline 或 find 返回的 heading ref 原样传给 read
- **THEN** read 返回对应 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: Canonical ref 与当前结构不匹配
- **WHEN** ref 符合 canonical grammar
- **AND** line 或 level 任一字段无法匹配当前解析结果中的同一 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown read 必须区分非法 ref grammar 与合法 ref 未匹配
`docnav-markdown read` MUST 将不符合 Markdown 当前 ref grammar 的非空 ref 映射为 `REF_INVALID`。错误 details MUST 包含原始 `ref` 和非空 `reason`。

符合 canonical heading grammar 但当前解析结果中没有匹配项的 ref MUST 返回 `REF_NOT_FOUND`，不得返回 `REF_INVALID`。旧 `H:L{line}:H{level}:I{index}` 格式不构成独立 ref 类型或兼容分支，只作为非法 grammar 的测试输入。

#### Scenario: 非 canonical heading ref 返回 REF_INVALID
- **WHEN** 调用方传入 `L5:Guide > Install`、`L9#2:Repeat`、`L1#1:Guide`、`H:L1:H1:I1`、带前导零的数字、缺少字段或未知 ref 类型
- **AND** 该值不匹配 Markdown adapter 定义的其它合法 ref
- **THEN** read 返回 `REF_INVALID`
- **THEN** error details 包含原始 `ref` 和非空 `reason`

#### Scenario: Canonical ref 未匹配返回 REF_NOT_FOUND
- **WHEN** 调用方传入符合 canonical grammar 的 heading ref
- **AND** 当前解析结果没有 line 和 level 全部匹配的 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown heading ref 必须明确采用结构快照语义
`docnav-markdown` MUST 将 heading ref 定义为生成时解析结果中的结构坐标，不得将其描述为 heading title、section 内容或文档版本的持久身份。

文档内容或 parser 结果变化后，同一个格式合法的 ref MAY 不再匹配、MAY 匹配当前结构中的另一个 heading，也 MAY 在结构坐标未变化时继续匹配。调用方获取当前结构时 MUST 使用当前 outline 或 find 返回的 ref；规范不得要求调用方预先检测文档是否变化，也不得保证旧 ref 一定失败。

#### Scenario: 文档变化后 ref 不提供身份保证
- **WHEN** heading title 或文档结构在 ref 生成后发生变化
- **THEN** read 只按当前解析结果中的 line 和 level 执行匹配
- **THEN** 结果不以旧 title、旧 breadcrumb、旧全文 heading index 或旧 section 内容作为身份校验条件
