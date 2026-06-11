本 spec delta 仍处于审核阶段。

## ADDED Requirements

### Requirement: Markdown heading ref 必须使用带字段标识的 canonical 格式
`docnav-markdown` MUST 为 heading 生成 `H:L{line}:H{level}:I{index}` 格式的 canonical ref。

- 首个 `H` MUST 标识 heading ref 类型。
- `L{line}` 中的 `line` MUST 是 heading 的 1-based 起始行号。
- `H{level}` 中的 `level` MUST 是 `1` 到 `6` 的 Markdown heading level。
- `I{index}` 中的 `index` MUST 是 heading 在全文有效 headings 中的 1-based 顺序号，并在可见性过滤前确定。
- 三个字段 MUST 使用不带前导零的十进制表示。

canonical ref MUST 匹配 `^H:L([1-9][0-9]*):H([1-6]):I([1-9][0-9]*)$`。ref 的生成 MUST 独立于 heading title、breadcrumb 和字符集。

#### Scenario: 生成 canonical heading ref
- **WHEN** 第 1 个有效 heading 是位于第 1 行的 H1
- **AND** 第 2 个有效 heading 是位于第 5 行的 H2
- **THEN** outline 分别输出 `H:L1:H1:I1` 和 `H:L5:H2:I2`

#### Scenario: Heading index 在过滤前确定
- **WHEN** 全文依次包含 H1、H4 和 H2
- **AND** 当前 `max_heading_level` 过滤掉 H4
- **THEN** H2 的 ref 仍使用 `index: 3`
- **THEN** outline 和 find 对该 H2 返回相同 ref

#### Scenario: 文本内容不进入 heading ref
- **WHEN** heading 包含极长标题、深层 breadcrumb 或 Unicode 文本
- **THEN** ref 仍只由 heading 类型、`L{line}`、`H{level}` 和 `I{index}` 构成

#### Scenario: 重复 heading 生成不同 ref
- **WHEN** 文档包含重复 title 或重复 breadcrumb
- **THEN** 每个 heading 根据自身 line、level 和 index 获得不同 ref

### Requirement: Markdown read 必须精确消费 canonical heading ref
`docnav-markdown read` MUST 解析 `H:L{line}:H{level}:I{index}`，并在当前文档解析结果中匹配三个字段全部相同的 heading。无匹配时 MUST 返回 `REF_NOT_FOUND`。

`read` MUST 将 heading ref 作为结构定位值，不使用 heading title、breadcrumb、section 内容或其摘要补充匹配。

#### Scenario: 读取 canonical heading ref
- **WHEN** 调用方把 outline 或 find 返回的 heading ref 原样传给 read
- **THEN** read 返回对应 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: Heading ref 与当前结构不匹配
- **WHEN** line、level 或 index 任一字段无法匹配同一 heading
- **THEN** read 返回 `REF_NOT_FOUND`

#### Scenario: 拒绝旧 heading ref
- **WHEN** 调用方传入 `L5:Guide > Install`、`L9#2:Repeat` 或 `L1#1:Guide`
- **THEN** read 返回稳定 ref 错误

### Requirement: Markdown adapter 必须保留整篇文档 ref
当当前 outline 参数下没有可见 heading 时，`docnav-markdown` MUST 返回单条 `doc:full` entry。`read` MUST 接受 `doc:full` 并返回整篇 Markdown 文档。

`doc:full` MUST 作为 Markdown adapter 私有 ref 处理，不属于 heading ref 格式。

#### Scenario: 无可见 heading 时读取整篇文档
- **WHEN** 当前 outline 参数过滤后没有可见 heading
- **THEN** outline 返回 ref 为 `doc:full` 的单条 entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown ref 的保证范围必须限于当前文档内容
`docnav-markdown` MUST 将 heading ref 定义为当前文档解析结果中的结构定位值。文档内容或 parser 结果变化后，调用方 MUST 重新执行 `outline` 或 `find` 获取 ref。

该契约 MUST 以 line、level 和 index 的结构匹配为边界；内容摘要、mtime 和版本标识不属于 Markdown ref 的生成或读取规则。

#### Scenario: 文档内容发生变化
- **WHEN** 调用方准备读取已经发生变化的 Markdown 文档
- **THEN** 调用方重新执行 outline 或 find 获取当前 ref

## REMOVED Requirements

### Requirement: Markdown heading ref 必须使用 canonical line-ordinal-path 格式
**Reason**: 该格式将完整 heading breadcrumb 编入 ref，使 ref 长度受标题文本控制。

**Migration**: 调用方重新执行 `outline` 或 `find` 获取 `H:L{line}:H{level}:I{index}`；Markdown 私有 ref `doc:full` 保持不变。
