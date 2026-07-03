# markdown-navigation Specification

## Purpose
定义 Markdown 导航能力，包括 Markdown adapter 的 manifest metadata、probe、outline、read、find、info、ref handling、pagination、core CLI output 和边界案例验证。
## Requirements
### Requirement: Markdown adapter 必须提供完整 v0 文档操作
Core-linked Markdown adapter MUST expose manifest metadata through its static descriptor and `docnav-adapter-contracts` handle，并声明 Markdown 格式身份、扩展名和 content type。`outline`、`read`、`find`、`info` 的固定文档操作面由 adapter contract 和 linked handler methods 定义。Manifest 字段集合 MUST 排除协议范围字段和 `recommended_parameters`，且不声明默认参数、native option values 或文档操作集合。

#### Scenario: 读取 linked adapter metadata
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** 输出包含 core release static registry 中的 Markdown adapter metadata
- **THEN** manifest 字段集合不包含 `protocol.min` 或 `protocol.max`
- **THEN** manifest 字段集合不包含 `recommended_parameters`

### Requirement: probe 必须只识别 Markdown 格式
Linked Markdown adapter probe MUST 只执行格式识别并返回支持度、格式 id 和判断证据，MUST NOT 执行 outline/read/find 导航；content type MUST 由 manifest metadata 声明。

#### Scenario: probe Markdown 文件
- **WHEN** 调用方对 Markdown 文档执行 probe
- **THEN** probe 返回 Markdown 格式证据
- **THEN** 不返回 outline entries 或 read content

### Requirement: outline 必须返回扁平且有限的 entries
Markdown outline MUST 按文档顺序返回扁平 entries，每条 entry MUST 包含完整 ref 和紧凑 display；默认只展示 H1-H3，默认字符预算为 6000。

#### Scenario: 嵌套 heading
- **WHEN** Markdown 文档包含 H1、H2 和 H3
- **THEN** outline 返回按文档顺序排列的扁平 entries
- **THEN** 每条 entry 包含 adapter 生成的唯一 ref

#### Scenario: 代码围栏伪 heading
- **WHEN** 代码围栏内包含看似 heading 的文本
- **THEN** outline 不把该文本作为 heading entry

#### Scenario: 当前 outline 为空
- **WHEN** 当前 outline 参数过滤后没有任何 heading entry
- **THEN** outline 返回一个全文 ref entry
- **THEN** 该 ref 可由 read 读取整篇 Markdown 文档

### Requirement: read 必须通过 ref 唯一读取 Markdown 区域
Markdown read MUST 解析 adapter 生成的 ref 并读取唯一文档区域；无匹配返回 `REF_NOT_FOUND`，多匹配返回 `REF_AMBIGUOUS`，MUST NOT 静默使用最近位置或首个匹配。

#### Scenario: 从 outline ref 读取章节
- **WHEN** 调用方将 outline 返回的 ref 原样传给 read
- **THEN** read 返回对应章节内容
- **THEN** content_type 为 `text/markdown`

#### Scenario: 重复完整 heading path
- **WHEN** 文档包含重复完整 heading path
- **THEN** outline 为每个重复项生成不同 ref
- **THEN** read 可通过每个 ref 分别定位对应区域

#### Scenario: 读取全文 ref
- **WHEN** 调用方将 outline 返回的全文 ref 原样传给 read
- **THEN** read 返回整篇 Markdown 文档
- **THEN** content_type 为 `text/markdown`

### Requirement: find 必须返回有限匹配并可继续
Markdown find MUST 按 query 搜索 Markdown 文档并返回 matches，每个 match MUST 包含 ref 和 display，结果 MUST 遵守 `limit` 和 page。match 的 ref MUST 指向当前 outline 参数下离命中位置最近的 outline entry；当当前 outline 为空时，match 的 ref MUST 指向全文 ref。

#### Scenario: find 返回下一页
- **WHEN** 匹配结果超过字符预算
- **THEN** find 只返回当前页预算内的 matches
- **THEN** 响应 page 为下一页页码

#### Scenario: find 归属到最近 outline
- **WHEN** query 命中文档中两个 outline entry 之间的内容
- **THEN** match ref 指向离命中位置最近的 outline entry
- **THEN** find 不把 match 默认归到全文 ref

### Requirement: info 必须返回 Markdown 紧凑摘要
Markdown info MUST 返回格式原生的紧凑摘要，至少表达格式身份、文档事实和 adapter 可读摘要。

#### Scenario: info Markdown 文档
- **WHEN** 调用方执行 Markdown info
- **THEN** 结果包含 Markdown content type
- **THEN** 结果表达 Markdown adapter 的可读摘要

### Requirement: Markdown 分页必须按 Unicode 字符预算
Markdown outline、read 和 find MUST 按 UTF-8 解码后的 Unicode 字符计数分页，MUST 保证 page 可继续，且 MUST 不切断 Unicode 字符。

#### Scenario: read 达到字符预算
- **WHEN** 章节内容超过 `limit`
- **THEN** read 返回当前页内容和下一页 page
- **THEN** 使用相同 ref 和下一页 page 可继续读取

### Requirement: Markdown 边界案例必须自动化验证
Markdown adapter 测试 MUST 覆盖无 heading、仅深层 heading、无效 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节和非 UTF-8。

#### Scenario: 运行 Markdown adapter 测试
- **WHEN** 实现者运行 adapter 测试
- **THEN** 全部参考边界案例都有对应测试或 fixture

### Requirement: Markdown adapter 必须通过 core CLI 黑盒 smoke 测试
Core CLI smoke MUST cover linked Markdown adapter behavior through the `docnav` executable. 测试必须启动构建后的 core binary，并通过真实进程边界传入 argv、cwd 和环境；Markdown adapter implementation source MUST come from the current core release static registry。核心 fixtures 必须是提交到项目中的固定文件。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、core adapter inspection、CLI help、linked adapter dispatch 和 strict argv failure path。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Strict input 行为：unknown argv、多余 positional、operation-inapplicable 参数和 undeclared native options 返回 primary `DiagnosticRecord` 投影。
- Readable-view framing：合法 JSON header、静态 `/content` block 引用、UTF-8 byte length、block 起止行和正文原值还原。

#### Scenario: Strict argv failure 被覆盖
- **WHEN** smoke 测试执行 `docnav outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav outline --unknown <path> --output readable-view`
- **OR** 执行 `docnav outline <path> --unknown --output protocol-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令返回 strict input failure
- **THEN** linked Markdown handler 不执行
- **THEN** failure output 投影 primary `DiagnosticRecord`

#### Scenario: 成功 smoke 使用成功 payload shape
- **WHEN** smoke suite 检查有效 `readable-json`、`readable-view` 和 `protocol-json` 输出
- **THEN** Markdown adapter behavior 通过 core-linked adapter dispatch 观察
- **THEN** successful document output 使用 owning success payload shape
- **THEN** readable success output 使用 current success schema fields
- **THEN** protocol JSON 输出符合 protocol response envelope 结构

### Requirement: Core Markdown smoke 必须输出可审计日志
Core CLI Markdown smoke runner MUST write an audit log for every executed command. The log MUST include the command line, working directory, fixture reference, exit code, stdout, stderr, and assertion summary. The runner MUST write a stable latest log and a timestamped log under `.log/smoke/core/`.

#### Scenario: 每条命令都有日志记录
- **WHEN** Node.js runner 执行任意正向或负向 CLI case
- **THEN** 日志记录该 case 的名称、命令行、cwd、exit code、stdout、stderr 和断言结果
- **THEN** 日志记录 Markdown fixture 引用

#### Scenario: 测试结束输出日志路径
- **WHEN** Node.js runner 完成 smoke suite
- **THEN** 终端摘要包含通过/失败状态和 `.log/smoke/core/latest.log` 路径
- **THEN** 完整命令输出可从 latest log 或时间戳日志复查

#### Scenario: 日志不记录无关环境信息
- **WHEN** Node.js runner 写入审计日志
- **THEN** 日志只包含测试命令、fixture 路径、stdout、stderr、exit code 和断言结果
- **THEN** 日志不转储完整环境变量或与测试无关的机器信息

### Requirement: Markdown adapter 必须有 core CLI 负向矩阵测试
Core CLI matrix MUST cover Markdown document operations through linked adapter dispatch, including invalid command-line input、strict argv failure、非法配置输入和 navigation input resolution native option validation。每个用例必须按所属输出层断言 stdout、stderr 和 process exit code。

矩阵必须覆盖：

- 必需语义：缺 path、缺 `--ref`、缺 `--query`。
- Strict argv：unknown flag、多余 positional、当前 operation 不使用的参数，包括值非法但未被当前 operation 使用的 known 参数。
- 实际使用参数失败：`page` 或 `limit` 为 0、`page` 或 `limit` 非数字、`output` 非法、`max_heading_level` 越界。
- 配置输入失败：present config invalid JSON、non-object root、未知顶层字段、未知 `defaults` 字段和 undeclared `options` key。
- 业务和输入错误：missing file、invalid ref、non-UTF-8 document。
- Protocol-shaped failure：`protocol-json` output mode must use the protocol failure envelope when strict argv、config input or native option validation fails.

#### Scenario: unknown argv 阻断 document execution
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的参数
- **AND** 当前 operation 的必需语义参数仍可被解析
- **THEN** 进程非零退出
- **THEN** stdout 按所选 output mode 承载 failure projection 或保持该错误路径允许的空 stdout
- **THEN** failure diagnostic 标出输入位置、received token、expected shape 和 guidance

#### Scenario: readable-json strict failure 使用 readable error
- **WHEN** CLI 矩阵以 `--output readable-json` 执行 strict argv failure 或 config input failure
- **THEN** stdout 输出 readable error payload
- **THEN** payload 投影一个 primary `DiagnosticRecord`
- **THEN** stdout 不包含 successful operation payload

#### Scenario: protocol-shaped stdout 使用 failure envelope
- **WHEN** CLI 矩阵以 `--output protocol-json` 执行 strict argv failure、config input failure 或 native option validation failure
- **THEN** stdout 通过 protocol response schema
- **THEN** failure envelope 投影一个 primary `DiagnosticRecord`
- **THEN** linked Markdown handler does not execute when failure belongs to core input/config boundary

### Requirement: 保留成熟 parser 行为基线
Markdown 适配器 MUST 使用成熟 parser；章节 MUST 从目标 heading 开始，并在下一个同级或更高级 heading 前结束。

#### Scenario: 读取包含子章节的章节
- **WHEN** read 选择包含更深层 heading 的章节
- **THEN** 结果包含子章节并在下一个同级或更高级 heading 前结束

### Requirement: Markdown Outline 扁平且有限
Markdown outline MUST 返回扁平 ref/display entries，内置默认 MUST 为每页最多 6000 字符且只展示 H1-H3。

#### Scenario: 嵌套 heading
- **WHEN** 文档包含 H1、H2 和 H3
- **THEN** outline 返回按文档顺序排列的扁平条目
- **THEN** 每项 ref 使用 Markdown heading path 表达层级
- **THEN** display 只保留 ref 之外的紧凑信息

### Requirement: Markdown Read 有限且可继续
Markdown read 内置默认 MUST 为每页最多 6000 字符，并 MUST 返回下一页 page 或 null。

#### Scenario: Read 超过默认限制
- **WHEN** 章节超过默认字符预算
- **THEN** read 返回有限内容和下一页 page

### Requirement: 重复项生成唯一 Ref
重复标题和重复完整路径 MUST 生成不同 ref；read MUST NOT 通过最近行静默消歧。

#### Scenario: 重复完整路径
- **WHEN** 文档包含重复完整 heading path
- **THEN** outline 为每项生成不同 ref

### Requirement: 复用 Markdown 边界案例
Markdown 适配器测试 MUST 覆盖无 heading、仅深层 heading、无效 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节和非 UTF-8。

#### Scenario: 规划适配器测试
- **WHEN** 实现者制定或更新 Markdown 测试计划
- **THEN** 测试计划包含全部参考边界案例

### Requirement: Markdown heading ref 必须使用带字段标识的 canonical 格式
`docnav-markdown` MUST 为 heading 生成 `H:L{line}:H{level}` 格式的 canonical ref。

- 首个 `H` MUST 标识 heading ref 类型。
- `L{line}` 中的 `line` MUST 是 heading 的 1-based 起始行号。
- `H{level}` 中的 `level` MUST 是 `1` 到 `6` 的 Markdown heading level。
- 两个数值字段 MUST 使用首位为 `1`–`9` 的十进制表示。

canonical ref MUST 匹配 `^H:L([1-9][0-9]*):H([1-6])$`。ref MUST 由 heading type、line 和 level 生成。ref 长度 MUST 由 line 的十进制位数决定。

#### Scenario: 生成 canonical heading ref
- **WHEN** 第 1 个有效 heading 是位于第 1 行的 H1
- **AND** 第 2 个有效 heading 是位于第 5 行的 H2
- **THEN** outline 分别输出 `H:L1:H1` 和 `H:L5:H2`

#### Scenario: 可见性过滤保持同一 heading 的 ref 稳定
- **WHEN** 全文依次包含 H1、H4 和 H2
- **AND** 当前 `max_heading_level` 过滤掉 H4
- **THEN** H2 的 ref 使用自身 line 和 level 生成的 `H:L{line}:H2`
- **THEN** outline 和 find 对该 H2 返回相同 ref

#### Scenario: canonical ref 由结构字段决定
- **WHEN** heading title、所属 breadcrumb 或 Unicode 文本很长
- **THEN** ref 仍只由 heading 类型、`L{line}` 和 `H{level}` 构成

#### Scenario: 重复 heading 生成不同 ref
- **WHEN** 文档包含重复 title 或重复 breadcrumb 且这些 heading 位于不同行
- **THEN** 每个 heading 根据自身 line 和 level 获得不同 ref

### Requirement: Markdown outline 和 find 必须在 display 中保留各自的可读语义
`docnav-markdown` MUST 通过 outline 的 `display` 提供 heading title 或 breadcrumb 导航语义。outline display MAY 同时包含 heading level、section cost 或其它紧凑摘要。

find 的 `display` MUST 保留匹配位置附近的非空文本片段，并 MAY 补充对应 heading 的 title 或 breadcrumb。

outline 的超长 title 或 breadcrumb，以及 find 的超长匹配片段或补充导航文本，MAY 按字符预算截断。截断后 MUST 保留该 operation 所需的非空核心语义；发生省略时 MUST 包含显式截断标记。截断 MUST 只影响 display；完整 ref MUST 始终由 `ref` 字段承载。read MUST 使用 `ref` 字段解析和定位 heading。

#### Scenario: Ref 与 display 分离职责
- **WHEN** outline 返回 heading entry
- **THEN** ref 使用 `H:L{line}:H{level}`
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
`docnav read <path> --ref <ref>` 经 core static registry dispatch 到 linked Markdown adapter 后，Markdown read handler MUST 解析 `H:L{line}:H{level}`，并在当前文档解析结果中匹配 line 和 level 全部相同的 heading。匹配成功时 MUST 返回该 heading 的当前 Markdown section；没有匹配项时 MUST 返回 `REF_NOT_FOUND`。

该精确匹配和当前解析结果中的唯一性 MUST 属于 Markdown adapter 私有行为。共享层 MUST 继续把 ref 作为 opaque string 原样传递。`read` MUST 使用 line 和 level 作为 heading 身份输入。

#### Scenario: 读取 canonical heading ref
- **WHEN** 调用方把当前 outline 或 find 返回的 heading ref 原样传给 read
- **THEN** read 返回对应 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: Canonical ref 与当前结构不匹配
- **WHEN** ref 符合 canonical grammar
- **AND** line 或 level 任一字段无法匹配当前解析结果中的同一 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown read 必须区分非法 ref grammar 与合法 ref 未匹配
`docnav read <path> --ref <ref>` 经 core static registry dispatch 到 linked Markdown adapter 后，Markdown read handler MUST 将当前合法 ref grammar 之外的非空 ref 映射为 `REF_INVALID`。错误 details MUST 包含原始 `ref` 和非空 `reason`。

符合 canonical heading grammar 但当前解析结果中没有匹配项的 ref MUST 返回 `REF_NOT_FOUND`。

#### Scenario: 非 canonical heading ref 返回 REF_INVALID
- **WHEN** 调用方传入当前合法 ref grammar 之外的非空 ref
- **THEN** read 返回 `REF_INVALID`
- **THEN** error details 包含原始 `ref` 和非空 `reason`

#### Scenario: Canonical ref 未匹配返回 REF_NOT_FOUND
- **WHEN** 调用方传入符合 canonical grammar 的 heading ref
- **AND** 当前解析结果没有 line 和 level 全部匹配的 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown adapter 必须保留整篇文档 ref
当当前 outline 参数下没有可见 heading 时，`docnav-markdown` MUST 返回单条 `doc:full` entry。`read` MUST 接受 `doc:full` 并返回整篇 Markdown 文档。

`doc:full` MUST 作为 Markdown adapter 私有 ref 处理，不属于 heading ref grammar。

#### Scenario: 无可见 heading 时读取整篇文档
- **WHEN** 当前 outline 参数过滤后没有可见 heading
- **THEN** outline 返回 ref 为 `doc:full` 的单条 entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown heading ref 必须明确采用结构快照语义
`docnav-markdown` MUST 将 heading ref 定义为生成时解析结果中的结构坐标。heading title、section 内容和文档版本属于 display、content 或外部状态。

文档内容或 parser 结果变化后，同一个格式合法的 ref MAY 返回 `REF_NOT_FOUND`、MAY 匹配当前结构中的另一个 heading，也 MAY 在结构坐标未变化时继续匹配。调用方获取当前结构时 MUST 使用当前 outline 或 find 返回的 ref；过期 ref 的结果由当前解析结果决定。

#### Scenario: 文档变化后 ref 仍按当前结构坐标解析
- **WHEN** heading title 或文档结构在 ref 生成后发生变化
- **THEN** read 按当前解析结果中的 line 和 level 执行匹配
- **THEN** heading 身份输入来自 ref 的结构坐标

### Requirement: Navigation input resolution 支持 Markdown native options
Markdown adapter MUST receive typed native option values through navigation input resolution. Core owns config source descriptor/path handoff; `docnav-navigation` owns raw config source discovery, JSON reading, source priority, selected adapter typed-field validation/extraction and config input diagnostics. Markdown adapter documents the business effect of validated options, such as `options.max_heading_level` controlling visible heading granularity.

Missing default config sources mean the corresponding layer has no config source. Present invalid config sources such as unreadable, invalid JSON or non-object root MUST fail at the navigation config source boundary. Unknown top-level fields, unknown `defaults` fields and undeclared `options` keys MUST fail before handler execution. Declared `options` keys enter selected-adapter typed-field validation/extraction with their source metadata preserved.

#### Scenario: 默认配置来源缺失表示 absence
- **WHEN** 项目级或用户级默认配置来源不存在
- **THEN** 该配置 source absent
- **THEN** Markdown adapter 使用其它有效来源和内置默认值继续构造参数来源

#### Scenario: present config invalid 时失败
- **WHEN** 默认配置路径存在
- **AND** 配置内容 unreadable、invalid JSON、non-object root、包含未知顶层字段或未知 `defaults` 字段
- **THEN** CLI 返回配置输入错误
- **THEN** 已知字段不被用于继续成功路径

#### Scenario: options key 由 navigation input resolution 校验
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** `docnav-navigation` 将 `options` object 合并为 selected Markdown native option source
- **THEN** selected-adapter typed-field validation/extraction 校验并提取 `max_heading_level`
- **WHEN** 配置文件包含 undeclared `options` key
- **THEN** CLI 返回 native option input diagnostic

### Requirement: Markdown native option config sources 必须由 core smoke 和矩阵测试覆盖
Docnav core smoke 和矩阵 MUST 覆盖 navigation-owned config source loading、优先级、配置 source absence、配置 input failure、native option metadata handoff，以及 navigation input resolution/default sources 进入 request construction 前补全过程的边界。

#### Scenario: 配置 source absence 被覆盖
- **WHEN** 默认项目级或用户级配置文件不存在
- **THEN** smoke 测试证明该 source absent
- **THEN** operation 可以使用其它有效来源和内置默认值成功

#### Scenario: 配置 input failure 被覆盖
- **WHEN** present config invalid JSON、non-object root、未知字段或 undeclared option key 出现
- **THEN** 矩阵测试证明 CLI 返回 strict failure
- **THEN** failure output 投影一个 primary `DiagnosticRecord`

#### Scenario: Valid options 仍影响 Markdown 导航语义
- **WHEN** 项目级配置来源设置 `options.max_heading_level`
- **AND** 该 key 由 selected Markdown typed-field declaration 声明并通过 navigation input resolution 校验
- **THEN** 测试证明 validated `options.max_heading_level` typed value 影响 `outline` 和 `find` 的 visible heading granularity
- **THEN** core operation 通过同一 navigation input resolution 边界补全后进入 handler

### Requirement: Markdown native options 提供 core 配置 schema 和示例参考
`docs/schemas/docnav-markdown-config.schema.json` MUST 描述 core `docnav` 配置 source 中 Markdown-relevant fields 的参考 shape，包含 `defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` 和 raw `options` native option map。`docs/examples/json/docnav-markdown-config.json` MUST 提供包含 `options.max_heading_level` 的 core config 示例。该 schema/example 用于文档校验和编辑器提示；runtime defaults、source-level native option registry entries 和 config validation ownership 由 navigation input resolution 与 adapter owner 文档定义。

#### Scenario: 配置示例通过 schema 校验
- **WHEN** docs validator 校验 `docs/examples/json/docnav-markdown-config.json`
- **THEN** 示例符合 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** schema 约束 `defaults.pagination.limit` 为正整数、`defaults.output` 为 core document output mode，并把 `options` 作为 raw native option map
- **THEN** navigation input resolution 通过 selected Markdown typed-field declaration 校验 `options.max_heading_level` 的 type 和 `1..6` 语义范围

#### Scenario: schema 不改变 core runtime 行为
- **WHEN** core CLI 读取 `docnav` core config
- **THEN** runtime 不要求加载 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** 配置读取和 source merge 由 navigation input resolution 与 source-level native option registry 负责
- **THEN** Markdown option 语义和 range validation 由 selected Markdown typed-field declaration 负责

### Requirement: Markdown linked adapter consumes resolved pagination defaults
Document operations MUST consume `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>` through navigation input resolution before dispatching the linked Markdown adapter. Markdown-specific code MUST keep ownership of Markdown's adapter-specific interpretation of `limit`.

#### Scenario: Core resolves pagination before Markdown dispatch
- **WHEN** core CLI runs a paginated Markdown document operation
- **THEN** navigation input resolution resolves the final limit and page before Markdown operation logic runs
- **THEN** Markdown-specific code keeps ownership of Markdown's budget-unit interpretation

#### Scenario: Markdown native option config example uses pagination limit
- **WHEN** core config schema or example for Markdown native options documents pagination defaults
- **THEN** it uses `defaults.pagination.enabled` for the default pagination state
- **THEN** it uses `defaults.pagination.limit` for the numeric budget default
- **THEN** it does not describe that budget as a core or navigation-input-resolution unit
- **THEN** any Markdown-specific unit description remains owned by the Markdown adapter documentation

