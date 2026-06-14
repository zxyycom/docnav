# markdown-navigation Specification

## Purpose
定义 Markdown 导航能力，包括 Markdown adapter 的 manifest、probe、outline、read、find、info、ref handling、pagination、direct CLI output、边界案例验证，以及 MarkdownNavigator 参考基线的来源和复验要求。
## Requirements
### Requirement: Markdown adapter 必须声明完整 v0 能力
`docnav-markdown` MUST 输出符合当前 manifest schema 的 manifest，并声明 Markdown 格式身份、扩展名、content type，以及 `outline`、`read`、`find`、`info` 全部 capability。Manifest 字段集合 MUST 排除协议范围字段和 `recommended_parameters`。

#### Scenario: 读取 manifest
- **WHEN** 调用方执行 `docnav-markdown manifest --output protocol-json`
- **THEN** 输出通过 manifest schema
- **THEN** capabilities 包含 `outline`、`read`、`find` 和 `info`
- **THEN** manifest 字段集合不包含 `protocol.min` 或 `protocol.max`
- **THEN** manifest 字段集合不包含 `recommended_parameters`

### Requirement: probe 必须只识别 Markdown 格式
`docnav-markdown probe` MUST 只执行格式识别并返回支持度、格式 id 和判断证据，MUST NOT 执行 outline/read/find 导航；content type MUST 由 manifest 声明。

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
Markdown find MUST 按 query 搜索 Markdown 文档并返回 matches，每个 match MUST 包含 ref 和 display，结果 MUST 遵守 `limit_chars` 和 page。match 的 ref MUST 指向当前 outline 参数下离命中位置最近的 outline entry；当当前 outline 为空时，match 的 ref MUST 指向全文 ref。

#### Scenario: find 返回下一页
- **WHEN** 匹配结果超过字符预算
- **THEN** find 只返回当前页预算内的 matches
- **THEN** 响应 page 为下一页页码

#### Scenario: find 归属到最近 outline
- **WHEN** query 命中文档中两个 outline entry 之间的内容
- **THEN** match ref 指向离命中位置最近的 outline entry
- **THEN** find 不把 match 默认归到全文 ref

### Requirement: info 必须返回 Markdown 紧凑摘要
Markdown info MUST 返回格式原生的紧凑摘要，至少表达格式身份、能力集合和 adapter 可读摘要。

#### Scenario: info Markdown 文档
- **WHEN** 调用方执行 Markdown info
- **THEN** 结果包含 Markdown content type
- **THEN** 结果表达 adapter 支持的 capability 集合

### Requirement: Markdown 分页必须按 Unicode 字符预算
Markdown outline、read 和 find MUST 按 UTF-8 解码后的 Unicode 字符计数分页，MUST 保证 page 可继续，且 MUST 不切断 Unicode 字符。

#### Scenario: read 达到字符预算
- **WHEN** 章节内容超过 `limit_chars`
- **THEN** read 返回当前页内容和下一页 page
- **THEN** 使用相同 ref 和下一页 page 可继续读取

### Requirement: Markdown 边界案例必须自动化验证
Markdown adapter 测试 MUST 覆盖无 heading、仅深层 heading、无效 heading、frontmatter、代码围栏、重复标题、重复路径、深层章节和非 UTF-8。

#### Scenario: 运行 Markdown adapter 测试
- **WHEN** 实现者运行 adapter 测试
- **THEN** 全部参考边界案例都有对应测试或 fixture

### Requirement: Markdown adapter 必须有完整黑盒 CLI smoke 测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI smoke 测试。测试必须启动构建后的 adapter binary，不得直接调用 adapter 内部 API。核心 fixtures 必须是提交到项目中的固定文件，不得在测试运行时临时生成。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、`probe`、`manifest`、有效 `invoke`、CLI help、direct CLI/invoke 共享语义归一和宽松 argv 成功路径。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Warning 行为：readable warning 使用稳定 envelope；CLI argv warning 使用 `id: "cli_argv_ignored"`；测试不断言 exact token 分组、`reason` 文案或 token 消费顺序。

#### Scenario: Node.js runner 使用构建产物
- **WHEN** smoke 测试运行
- **THEN** 测试使用已构建的 `docnav-markdown` binary 路径启动真实进程
- **THEN** Node.js runner 负责传入命令参数、stdin、工作目录和环境
- **THEN** 测试不通过 Rust adapter 内部 API 完成黑盒断言

#### Scenario: fixture corpus 是固定项目文件
- **WHEN** reviewer 查看 smoke fixture corpus
- **THEN** normal、duplicate headings、frontmatter、code fence、deep headings、no headings、unicode、large pagination、non-UTF-8、UTF-8 BOM、CRLF、`.MD` 和 `.markdown` 用例都能在项目目录中直接找到
- **THEN** 核心 fixture 内容不依赖测试运行时临时生成

#### Scenario: 通过 CLI outline ref 读取内容
- **WHEN** smoke 测试对 normal Markdown fixture 执行 `docnav-markdown outline <path> --output readable-json` 并提取 entry ref
- **THEN** 使用该 ref 执行 `docnav-markdown read <path> --ref <ref> --output readable-json` 返回对应 Markdown 内容
- **THEN** read 结果包含 `content_type: "text/markdown"`
- **THEN** outline 和 read 的 readable JSON 均不包含 protocol envelope 字段

#### Scenario: protocol-json smoke 使用 envelope
- **WHEN** smoke 测试执行 `docnav-markdown read <path> --ref <ref> --output protocol-json`
- **THEN** stdout 包含成功 protocol response envelope
- **THEN** envelope 的 `operation` 为 `read`
- **THEN** stderr 不包含用户可读结果或重复 JSON payload

#### Scenario: readable-view 输出 smoke 保留关键可读信息
- **WHEN** smoke 测试执行 `outline`、`read`、`find` 和 `info` 的 `readable-view` 输出
- **THEN** stdout 包含 pretty JSON header 和对应 operation 的关键可读信息，例如 ref、content block、content_type、cost、capabilities 或 page 状态
- **THEN** stdout 不包含完整 protocol envelope JSON
- **THEN** 成功路径 stderr 为空或只包含非阻断诊断

#### Scenario: CLI help 可用于纠错
- **WHEN** smoke 测试执行 `docnav-markdown --help`
- **OR** 执行 `docnav-markdown outline --help`
- **THEN** stdout 或 stderr 包含可用命令、关键参数、默认值或输出模式信息
- **THEN** 该命令不执行文档导航业务

#### Scenario: 宽松 argv 成功路径被覆盖
- **WHEN** smoke 测试执行 `docnav-markdown outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav-markdown outline --unknown <path> --output readable-json`
- **OR** 执行 `docnav-markdown outline <path> --unknown --output protocol-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令成功返回所选输出模式的正常结果
- **THEN** 输出包含 warning 或诊断
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

#### Scenario: fixture corpus 覆盖 Markdown 边界
- **WHEN** smoke corpus 被执行
- **THEN** 重复 heading 产生不同 ref
- **THEN** frontmatter 和代码围栏伪 heading 不产生 outline entry
- **THEN** 深层 heading 和无 heading fixture 在可见 outline 为空时可回退到全文 ref
- **THEN** Unicode 和 large pagination fixture 证明 page 可继续读取且不会切断 Unicode 字符
- **THEN** UTF-8 BOM 和 CRLF fixture 可被读取并保持正确 outline/read 行为
- **THEN** `.MD` 和 `.markdown` 扩展名 fixture 可被 probe 识别为 Markdown
- **THEN** 非 UTF-8 fixture 通过 CLI 返回稳定编码错误

#### Scenario: manifest probe find info 和 invoke 被覆盖
- **WHEN** smoke suite 执行 manifest、probe、find、info 和有效 invoke 请求
- **THEN** manifest 声明 Markdown capabilities 和 supported formats
- **THEN** probe 返回格式证据且不包含 outline/read payload
- **THEN** find 返回带 ref 和 page 状态的 matches
- **THEN** info 返回 Markdown 摘要和 capabilities
- **THEN** 有效 invoke 请求从 stdin 输入后在 stdout 返回成功 protocol envelope
- **THEN** direct CLI 与等价有效 invoke 请求经过共享语义归一或等价 request 构造后执行同一 operation 处理

#### Scenario: JSON 输出通过 schema 或等价结构校验
- **WHEN** smoke suite 检查 `readable-json` 和 `protocol-json` 输出
- **THEN** readable JSON 输出符合对应 readable schema 或等价字段集合断言
- **THEN** protocol JSON 输出符合 protocol response envelope 结构
- **THEN** manifest 和 probe 输出符合对应 manifest/probe 结构

#### Scenario: 分页继续和越界 page 被覆盖
- **WHEN** smoke suite 对 large pagination fixture 使用返回的下一页 page 继续读取
- **THEN** 第二页返回后续内容且 page 状态可继续或为 null
- **WHEN** smoke suite 请求超过结果末尾的 page
- **THEN** 返回空结果和 `page: null`，且不作为错误

### Requirement: Markdown CLI smoke 必须输出可审计日志
`docnav-markdown` black-box CLI smoke runner MUST write an audit log for every executed command. The log MUST include the command line, working directory, stdin summary or fixture reference, exit code, stdout, stderr, and assertion summary. The runner MUST write a stable latest log and a timestamped log under `.log/docnav-markdown-cli-smoke/`.

#### Scenario: 每条命令都有日志记录
- **WHEN** Node.js runner 执行任意正向或负向 CLI case
- **THEN** 日志记录该 case 的名称、命令行、cwd、exit code、stdout、stderr 和断言结果
- **THEN** 若命令使用 stdin，日志记录 stdin 的测试输入摘要或 fixture 引用

#### Scenario: 测试结束输出日志路径
- **WHEN** Node.js runner 完成 smoke suite
- **THEN** 终端摘要包含通过/失败状态和 `.log/docnav-markdown-cli-smoke/latest.log` 路径
- **THEN** 完整命令输出可从 latest log 或时间戳日志复查

#### Scenario: 日志不记录无关环境信息
- **WHEN** Node.js runner 写入审计日志
- **THEN** 日志只包含测试命令、fixture 路径、stdin 摘要、stdout、stderr、exit code 和断言结果
- **THEN** 日志不转储完整环境变量或与测试无关的机器信息

### Requirement: Markdown adapter 必须有负向 CLI 矩阵测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI 矩阵测试，覆盖非法命令行输入、宽松 argv 输入和非法 invoke 输入。每个用例必须按所属输出层断言 stdout、stderr 和 process exit code。

矩阵必须覆盖：

- 必需语义：缺 path、缺 `--ref`、缺 `--query`。
- 宽松 argv：unknown flag、多余 positional、当前 operation 不使用的参数，包括值非法但未被当前 operation 使用的 known 参数。
- 实际使用参数失败：`page` 或 `limit_chars` 为 0、`page` 或 `limit_chars` 非数字、`output` 非法、`max_heading_level` 越界。
- 业务和输入错误：missing file、invalid ref、non-UTF-8 document。
- Invoke 传输错误：malformed invoke JSON、缺少必需字段或参数类型错误等 schema-valid JSON shape 错误。
- Warning 断言：稳定 warning envelope 和输出通道边界；不断言 exact token 分组、`reason` 文案或 token 消费顺序。

#### Scenario: 参数校验失败保持 CLI 诊断
- **WHEN** 负向矩阵执行缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit、非法 output 或非法 max heading level
- **THEN** 进程非零退出
- **THEN** stderr 或所选错误输出包含简洁诊断
- **THEN** stdout 不包含成功的 protocol payload 或 readable result payload

#### Scenario: unknown argv 不阻断成功路径
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的参数
- **OR** 执行当前 operation 不使用、且值无法通过其它 operation 类型或范围校验的 known 参数
- **AND** 当前 operation 的必需语义参数仍可被解析
- **THEN** 进程成功退出
- **THEN** stdout 包含所选输出模式的正常结果
- **THEN** warning 或诊断必须存在
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

#### Scenario: readable-json warning envelope 保留
- **WHEN** CLI 矩阵以 `--output readable-json` 执行宽松 argv 成功路径
- **THEN** stdout 包含顶层 `warnings` 数组
- **THEN** 每个 warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** stdout 通过对应 readable schema

#### Scenario: protocol-shaped stdout 不承载 warning
- **WHEN** CLI 矩阵以 `protocol-json`、manifest 或 probe 输出模式执行宽松 argv 成功路径
- **THEN** stdout 通过对应 protocol、manifest 或 probe schema
- **THEN** stdout 不因为 CLI warning 增加 `warnings` 字段
- **THEN** warning 或诊断只允许出现在 stderr 或非 schema stdout 之外的通道

#### Scenario: 当前 operation 使用的已知参数仍严格校验
- **WHEN** 负向矩阵执行 `docnav-markdown outline <path> --page 0`
- **OR** 执行 `docnav-markdown outline <path> --limit-chars nope`
- **OR** 执行 `docnav-markdown outline <path> --output nope`
- **OR** 执行 `docnav-markdown outline <path> --max-heading-level 9`
- **THEN** 进程非零退出
- **THEN** 诊断指出对应已知参数非法

#### Scenario: readable operation 错误保留 code 和 details
- **WHEN** 负向矩阵以 `--output readable-json` 执行 missing file、invalid ref 或 non-UTF-8 document 用例
- **THEN** stdout 包含 readable error JSON，并保留稳定 `code`、`error`、`details` 和 `guidance`
- **THEN** stdout 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`
- **THEN** stderr 不包含替代 readable payload

#### Scenario: protocol-json operation 错误保留 envelope
- **WHEN** 负向矩阵以 `--output protocol-json` 执行 invalid ref 或 non-UTF-8 document 用例
- **THEN** stdout 包含 failure protocol envelope
- **THEN** envelope 保留 request operation 和稳定 error details
- **THEN** stderr 只包含诊断，且不重复 protocol JSON

#### Scenario: malformed invoke JSON 返回结构化协议失败
- **WHEN** 负向矩阵向 `docnav-markdown invoke` 写入 malformed JSON
- **THEN** stdout 包含 `operation: null` 且 error code 为 `INVALID_REQUEST` 的 protocol failure envelope
- **THEN** 进程非零退出

#### Scenario: invoke 参数 schema 错误返回结构化协议失败
- **WHEN** 负向矩阵向 `docnav-markdown invoke` 写入 JSON 语法合法但缺少必需字段或参数类型错误的请求
- **THEN** stdout 包含 `INVALID_REQUEST` protocol failure envelope
- **THEN** failure envelope 的 operation 在可解析时保留对应 operation，否则为 null
- **THEN** 进程非零退出

### Requirement: 记录 MarkdownNavigator 来源和复验方法
Markdown 参考文档 MUST 记录参考项目路径、提交、命令入口和可复验行为边界，并明确其不是 Docnav 兼容目标。

#### Scenario: 复验参考来源
- **WHEN** 实现者查看 Markdown 行为来源
- **THEN** 文档能够定位参考仓库、提交和复验方法

### Requirement: 记录 Markdown 行为迁移决策
Markdown 参考文档 MUST 为 heading、章节范围、frontmatter、代码围栏、重复项、编码、默认限制和 page 标注迁移决策。

#### Scenario: 评估旧行为
- **WHEN** 实现者查看参考 CLI 行为
- **THEN** 文档明确该行为在 Docnav 中保留、调整、推迟或移除

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
`docnav-markdown` MUST 为 heading 生成 `H:L{line}:H{level}:I{index}` 格式的 canonical ref。

- 首个 `H` MUST 标识 heading ref 类型。
- `L{line}` 中的 `line` MUST 是 heading 的 1-based 起始行号。
- `H{level}` 中的 `level` MUST 是 `1` 到 `6` 的 Markdown heading level。
- `I{index}` 中的 `index` MUST 是 heading 在全文有效 headings 中的 1-based 顺序号，并在可见性过滤前确定。
- 三个字段 MUST 使用不带前导零的十进制表示。

canonical ref MUST 匹配 `^H:L([1-9][0-9]*):H([1-6]):I([1-9][0-9]*)$`。ref 的生成 MUST 独立于 heading title、breadcrumb 和字符集。这里的长度保证 MUST 表达为不受标题文本和 breadcrumb 深度影响，不要求每个具体 heading 的新 ref 都短于旧格式。

#### Scenario: 生成 canonical heading ref
- **WHEN** 第 1 个有效 heading 是位于第 1 行的 H1
- **AND** 第 2 个有效 heading 是位于第 5 行的 H2
- **THEN** outline 分别输出 `H:L1:H1:I1` 和 `H:L5:H2:I2`

#### Scenario: Heading index 在过滤前确定
- **WHEN** 全文依次包含 H1、H4 和 H2
- **AND** 当前 `max_heading_level` 过滤掉 H4
- **THEN** H2 的 ref 使用 `H:L{line}:H2:I3`
- **THEN** outline 和 find 对该 H2 返回相同 ref

#### Scenario: 文本内容不进入 heading ref
- **WHEN** heading 包含极长标题、深层 breadcrumb 或 Unicode 文本
- **THEN** ref 仍只由 heading 类型、`L{line}`、`H{level}` 和 `I{index}` 构成

#### Scenario: 重复 heading 生成不同 ref
- **WHEN** 文档包含重复 title 或重复 breadcrumb
- **THEN** 每个 heading 根据自身 line、level 和 index 获得不同 ref

### Requirement: Markdown outline 和 find 必须在 display 中保留各自的可读语义
`docnav-markdown` MUST 在 heading ref 不包含标题文本时，通过 outline 的 `display` 提供 heading title 或 breadcrumb 导航语义。outline display MAY 同时包含 heading level、section cost 或其它紧凑摘要。

find 的 `display` MUST 保留匹配位置附近的非空文本片段，并 MAY 补充对应 heading 的 title 或 breadcrumb。find 不得为了补充 heading 导航语义而删除命中上下文。

outline 的超长 title 或 breadcrumb，以及 find 的超长匹配片段或补充导航文本，MAY 按字符预算截断。截断后 MUST 保留该 operation 所需的非空核心语义；发生省略时 MUST 包含显式截断标记。截断 MUST 只影响 display，不得修改 ref；完整 ref MUST 始终由 `ref` 字段承载。display MUST NOT 成为 read 解析 ref 或定位 heading 的输入。

#### Scenario: Ref 与 display 分离职责
- **WHEN** outline 返回 heading entry
- **THEN** ref 使用 `H:L{line}:H{level}:I{index}` 且不包含 title 或 breadcrumb
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
`docnav-markdown read` MUST 解析 `H:L{line}:H{level}:I{index}`，并在当前文档解析结果中匹配 line、level 和 index 全部相同的 heading。匹配成功时 MUST 返回该 heading 的当前 Markdown section；没有匹配项时 MUST 返回 `REF_NOT_FOUND`。

该精确匹配和当前解析结果中的唯一性 MUST 属于 Markdown adapter 私有行为，不得提升为所有 adapter 的共享 ref 保证。`read` MUST NOT 使用 heading title、breadcrumb、section 内容或其摘要补充匹配。

#### Scenario: 读取 canonical heading ref
- **WHEN** 调用方把当前 outline 或 find 返回的 heading ref 原样传给 read
- **THEN** read 返回对应 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: Canonical ref 与当前结构不匹配
- **WHEN** ref 符合 canonical grammar
- **AND** line、level 或 index 任一字段无法匹配当前解析结果中的同一 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown read 必须区分非法 ref grammar 与合法 ref 未匹配
`docnav-markdown read` MUST 将不符合 Markdown 当前 ref grammar 的非空 ref 映射为 `REF_INVALID`。错误 details MUST 包含原始 `ref` 和非空 `reason`。

符合 canonical heading grammar 但当前解析结果中没有匹配项的 ref MUST 返回 `REF_NOT_FOUND`，不得返回 `REF_INVALID`。旧格式不构成独立 ref 类型或兼容分支，只作为非法 grammar 的测试输入。

#### Scenario: 非 canonical heading ref 返回 REF_INVALID
- **WHEN** 调用方传入 `L5:Guide > Install`、`L9#2:Repeat`、`L1#1:Guide`、带前导零的数字、缺少字段或未知 ref 类型
- **AND** 该值不匹配 Markdown adapter 定义的其它合法 ref
- **THEN** read 返回 `REF_INVALID`
- **THEN** error details 包含原始 `ref` 和非空 `reason`

#### Scenario: Canonical ref 未匹配返回 REF_NOT_FOUND
- **WHEN** 调用方传入符合 canonical grammar 的 heading ref
- **AND** 当前解析结果没有 line、level 和 index 全部匹配的 heading
- **THEN** read 返回 `REF_NOT_FOUND`

### Requirement: Markdown adapter 必须保留整篇文档 ref
当当前 outline 参数下没有可见 heading 时，`docnav-markdown` MUST 返回单条 `doc:full` entry。`read` MUST 接受 `doc:full` 并返回整篇 Markdown 文档。

`doc:full` MUST 作为 Markdown adapter 私有 ref 处理，不属于 heading ref grammar。

#### Scenario: 无可见 heading 时读取整篇文档
- **WHEN** 当前 outline 参数过滤后没有可见 heading
- **THEN** outline 返回 ref 为 `doc:full` 的单条 entry
- **THEN** 使用该 ref 执行 read 返回整篇 Markdown 文档

### Requirement: Markdown heading ref 必须明确采用结构快照语义
`docnav-markdown` MUST 将 heading ref 定义为生成时解析结果中的结构坐标，不得将其描述为 heading title、section 内容或文档版本的持久身份。

文档内容或 parser 结果变化后，同一个格式合法的 ref MAY 不再匹配、MAY 匹配当前结构中的另一个 heading，也 MAY 在结构坐标未变化时继续匹配。调用方获取当前结构时 MUST 使用当前 outline 或 find 返回的 ref；规范不得要求调用方预先检测文档是否变化，也不得保证旧 ref 一定失败。

#### Scenario: 文档变化后 ref 不提供身份保证
- **WHEN** heading title 或文档结构在 ref 生成后发生变化
- **THEN** read 只按当前解析结果中的 line、level 和 index 执行匹配
- **THEN** 结果不以旧 title、旧 breadcrumb 或旧 section 内容作为身份校验条件
