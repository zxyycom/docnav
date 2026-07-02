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
Core CLI smoke MUST cover linked Markdown adapter behavior through the `docnav` executable. 测试必须启动构建后的 core binary，并通过真实进程边界传入 argv、cwd 和环境；Markdown adapter implementation source MUST come from the current core release static registry. 核心 fixtures 必须是提交到项目中的固定文件。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、core adapter inspection、CLI help、linked adapter dispatch 和 strict argv failure path。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Strict failure 行为：unknown argv、多余 positional 和当前 operation 不适用参数返回 primary input diagnostic。
- Readable-view framing：合法 JSON header、静态 `/content` block 引用、UTF-8 byte length、block 起止行和正文原值还原。

#### Scenario: Node.js runner 使用 core 构建产物
- **WHEN** smoke 测试运行
- **THEN** 测试使用已构建的 `docnav` binary 路径启动真实进程
- **THEN** Node.js runner 负责传入命令参数、工作目录和环境
- **THEN** 黑盒断言基于进程 stdout、stderr 和 exit code
- **THEN** Markdown adapter behavior 通过 core-linked adapter dispatch 观察

#### Scenario: fixture corpus 是固定项目文件
- **WHEN** reviewer 查看 smoke fixture corpus
- **THEN** normal、duplicate headings、frontmatter、code fence、deep headings、no headings、unicode、large pagination、non-UTF-8、UTF-8 BOM、CRLF、`.MD` 和 `.markdown` 用例都能在项目目录中直接找到
- **THEN** 核心 fixture 内容不依赖测试运行时临时生成

#### Scenario: 通过 CLI outline ref 读取内容
- **WHEN** smoke 测试对 normal Markdown fixture 执行 `docnav outline <path> --output readable-json` 并提取 entry ref
- **THEN** 使用该 ref 执行 `docnav read <path> --ref <ref> --output readable-json` 返回对应 Markdown 内容
- **THEN** read 结果包含 `content_type: "text/markdown"`
- **THEN** outline 和 read 的 readable JSON 均不包含 protocol envelope 字段

#### Scenario: protocol-json smoke 使用 envelope
- **WHEN** smoke 测试执行 `docnav read <path> --ref <ref> --output protocol-json`
- **THEN** stdout 包含成功 protocol response envelope
- **THEN** envelope 的 `operation` 为 `read`
- **THEN** stderr 不包含用户可读结果或重复 JSON payload

#### Scenario: readable-view 输出 smoke 保留完整字段和 Markdown block
- **WHEN** smoke 测试执行 `outline`、`read`、`find` 和 `info` 的默认或显式 `readable-view` 输出
- **THEN** 每个 stdout 从 pretty JSON header 开始
- **THEN** JSON header 包含对应 operation 的全部 readable 字段和 page 状态
- **THEN** read header 的 content 原位置包含 `/content` block 引用和 UTF-8 byte length
- **THEN** `/content` block 还原值等于 readable-json content
- **THEN** stdout 不包含完整 protocol envelope
- **THEN** 成功路径 stderr 为空或只包含非阻断诊断

#### Scenario: CLI help 可用于纠错
- **WHEN** smoke 测试执行 `docnav --help`
- **OR** 执行 `docnav outline --help`
- **THEN** stdout 或 stderr 包含可用命令、关键参数、默认值或输出模式信息
- **THEN** help 只把 readable-view、readable-json 和 protocol-json 列为 document operation 输出模式
- **THEN** 该命令不执行文档导航业务

#### Scenario: document output 值按三种模式校验
- **WHEN** smoke 测试执行 `docnav outline <path> --output <invalid-output>`
- **THEN** 命令非零退出并报告非法 output value
- **THEN** stdout 为空或仅包含该错误路径允许的诊断 payload

#### Scenario: adapter inspection 使用 core registry
- **WHEN** smoke 测试执行 `docnav adapter list`
- **THEN** 输出包含 linked Markdown adapter 的 id、version 和 formats
- **THEN** adapter inspection 不要求独立 Markdown executable

#### Scenario: strict argv failure path 被覆盖
- **WHEN** smoke 测试执行 `docnav outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav outline --unknown <path> --output readable-view`
- **OR** 执行 `docnav outline <path> --unknown --output protocol-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令非零退出
- **THEN** 输出包含 primary input diagnostic
- **THEN** 不返回所选输出模式的成功结果

#### Scenario: fixture corpus 覆盖 Markdown 边界
- **WHEN** smoke corpus 被执行
- **THEN** 重复 heading 产生不同 ref
- **THEN** frontmatter 和代码围栏伪 heading 不产生 outline entry
- **THEN** 深层 heading 和无 heading fixture 在可见 outline 为空时可回退到全文 ref
- **THEN** Unicode 和 large pagination fixture 证明 page 可继续读取且不会切断 Unicode 字符
- **THEN** UTF-8 BOM 和 CRLF fixture 可被读取并保持正确 outline/read 行为
- **THEN** `.MD` 和 `.markdown` 扩展名 fixture 可被 probe 识别为 Markdown
- **THEN** 非 UTF-8 fixture 通过 CLI 返回稳定编码错误

#### Scenario: registry metadata find info 被覆盖
- **WHEN** smoke suite 执行 adapter inspection、find 和 info
- **THEN** registry metadata 声明 Markdown supported formats
- **THEN** find 返回带 ref 和 page 状态的 matches
- **THEN** info 返回 Markdown 摘要
- **THEN** core CLI document operations 通过 linked Markdown adapter handler dispatch

#### Scenario: JSON 输出通过 schema 或等价结构校验
- **WHEN** smoke suite 检查 `readable-json` 和 `protocol-json` 输出
- **THEN** readable JSON 输出符合对应 readable schema 或等价字段集合断言
- **THEN** protocol JSON 输出符合 protocol response envelope 结构
- **THEN** adapter inspection 输出符合 core command owner 定义的 metadata shape

#### Scenario: 分页继续和越界 page 被覆盖
- **WHEN** smoke suite 对 large pagination fixture 使用返回的下一页 page 继续读取
- **THEN** 第二页返回后续内容且 page 状态可继续或为 null
- **WHEN** smoke suite 请求超过结果末尾的 page
- **THEN** 返回空结果和 `page: null`，且不作为错误

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
Core CLI smoke MUST provide a black-box matrix for Markdown document operations, covering invalid command-line input, strict argv input, invalid config input and Markdown adapter-owned option validation through linked adapter dispatch. 每个用例必须按所属输出层断言 stdout、stderr 和 process exit code。

矩阵必须覆盖：

- 必需语义：缺 path、缺 `--ref`、缺 `--query`。
- Strict argv：unknown flag、多余 positional、当前 operation 不使用的参数，包括值非法但未被当前 operation 使用的 known 参数。
- 实际使用参数失败：`page` 或 `limit` 为 0、`page` 或 `limit` 非数字、`output` 非法、`max_heading_level` 越界。
- 业务和输入错误：missing file、invalid ref、non-UTF-8 document。
- Protocol-json failure projection：CLI 输入、配置输入和 adapter-owned option validation failure 在 `--output protocol-json` 下产生 protocol-shaped failure envelope。
- Failure 断言：primary input diagnostic 和输出通道边界。

#### Scenario: 参数校验失败保持 CLI 诊断
- **WHEN** 负向矩阵执行缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit、非法 output 或非法 max heading level
- **THEN** 进程非零退出
- **THEN** stderr 或所选错误输出包含简洁诊断
- **THEN** stdout 不包含成功的 protocol payload 或 readable result payload

#### Scenario: unknown argv 阻断操作
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的参数
- **OR** 执行当前 operation 不使用、且值无法通过其它 operation 类型或范围校验的 known 参数
- **AND** 当前 operation 的必需语义参数仍可被解析
- **THEN** 进程非零退出
- **THEN** 输出包含 primary input diagnostic
- **THEN** stdout 不包含成功结果

#### Scenario: 当前 operation 使用的已知参数仍严格校验
- **WHEN** 负向矩阵执行 `docnav outline <path> --page 0`
- **OR** 执行 `docnav outline <path> --limit nope`
- **OR** 执行 `docnav outline <path> --output nope`
- **OR** 执行 `docnav outline <path> --max-heading-level 9`
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

#### Scenario: protocol-json input failure 返回结构化协议失败
- **WHEN** 负向矩阵以 `--output protocol-json` 执行 unknown argv、缺少必需参数或参数类型错误的 core CLI 请求
- **THEN** stdout 包含 `INVALID_REQUEST` protocol failure envelope
- **THEN** failure envelope 的 operation 在可解析时保留对应 operation，否则为 null
- **THEN** 失败来自标准参数/typed-field processing
- **THEN** 进程非零退出

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

### Requirement: Core CLI 支持 core 配置中的 Markdown native options
Core `docnav` document operations MUST read project/user core config through the standard parameter pipeline and source-level native option registry. Project config MUST be read from `<project-root>/.docnav/docnav.json`; user config MUST come from the core user config file. 首期 core 配置 MUST support `defaults.pagination.enabled`、`defaults.pagination.limit` 和 `defaults.output`；`options` MUST remain a raw adapter-owned native option map. 当前 Markdown static descriptor declares `options.max_heading_level` as a public native option source, and its type semantics and `1..6` range validation are owned by the Markdown adapter when consumed. Core MUST resolve project root/cwd/document path before adapter dispatch and pass an absolute document path to the navigation layer and selected Markdown adapter. Missing default config sources MUST be treated as absent; present invalid config sources MUST produce blocking config diagnostics.

#### Scenario: max_heading_level 来自配置
- **WHEN** `<project-root>/.docnav/docnav.json` 包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** outline 只显示当前 max_heading_level 下可见的 heading entries
- **THEN** 该结果与 protocol request `arguments.options.max_heading_level: 2` 的行为一致
- **THEN** adapter 接收的 document path 是 core 解析后的 absolute path

#### Scenario: find 使用配置中的 max_heading_level
- **WHEN** core 用户配置文件包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav find docs/guide.md --query install`
- **THEN** find 只搜索当前 max_heading_level 下可见的 heading entries
- **THEN** 该结果与 protocol request `arguments.options.max_heading_level: 2` 的行为一致

#### Scenario: 显式 max_heading_level 覆盖配置
- **WHEN** `<project-root>/.docnav/docnav.json` 包含 `options.max_heading_level: 2`
- **AND** 调用方执行 `docnav outline docs/guide.md --max-heading-level 4`
- **THEN** standard parameter pipeline 将最终 native option source 设为显式 argv 值 `4`
- **THEN** 配置值 `2` 不覆盖显式 argv

#### Scenario: 项目配置覆盖用户配置
- **WHEN** core 用户配置文件包含 `options.max_heading_level: 2`
- **AND** `<project-root>/.docnav/docnav.json` 包含 `options.max_heading_level: 4`
- **AND** 调用方执行 `docnav outline docs/guide.md`
- **THEN** standard parameter pipeline 将最终 native option source 设为 project config 值 `4`
- **THEN** user config 值 `2` 不覆盖 project config

#### Scenario: output 默认值来自配置
- **WHEN** `<project-root>/.docnav/docnav.json` 包含 `defaults.output: "readable-json"`
- **AND** 调用方执行 `docnav info docs/guide.md` 且未传入 `--output`
- **THEN** stdout 使用 readable-json 输出
- **THEN** 输出不使用 readable-view block framing

### Requirement: Markdown native option config sources 必须由 core smoke 和矩阵测试覆盖
Core CLI smoke 和矩阵 MUST 覆盖 core 配置读取、优先级、present invalid config source 返回 blocking config diagnostic，以及 protocol request `arguments` 与配置/default sources 进入同一标准参数解析的边界。

#### Scenario: Smoke 覆盖配置优先级
- **WHEN** smoke suite 使用项目级 `<project-root>/.docnav/docnav.json` 和 core 用户配置文件
- **THEN** 测试证明显式 argv 覆盖项目级配置
- **THEN** 项目级配置覆盖用户级配置
- **THEN** 用户级配置覆盖内置默认值
- **THEN** 测试证明 `outline` 和 `find` 都消费适用的 `options.max_heading_level`

#### Scenario: 矩阵覆盖配置源不可用
- **WHEN** smoke 或矩阵 fixture 提供语法无效的 JSON 配置源
- **AND** 其它配置来源或内置默认值可用
- **THEN** `docnav` 返回 blocking config diagnostic
- **THEN** operation request 不构造

#### Scenario: Protocol arguments 使用配置补足 registered option
- **WHEN** `<project-root>/.docnav/docnav.json` 设置 `options.max_heading_level`
- **AND** smoke suite 通过 core `docnav --output protocol-json` 执行未显式携带 options 的 outline request
- **THEN** core standard parameter pipeline 将 request `arguments` 映射为 direct input
- **THEN** pipeline 将项目级配置中的 `options.max_heading_level` 保留为 project config native option source
- **THEN** outline 行为与 request 显式携带相同 `max_heading_level` 的行为一致

### Requirement: Markdown native options 提供 core 配置 schema 和示例参考
`docs/schemas/docnav-markdown-config.schema.json` MUST 描述 core `docnav` 配置中 Markdown-relevant fields 的参考 shape，包含 `defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` 和 raw `options` native option map。`docs/examples/json/docnav-markdown-config.json` MUST 提供包含 `options.max_heading_level` 的 core config 示例。该 schema/example 用于文档校验和编辑器提示；runtime defaults、source-level native option registry entries 和 config validation ownership 由标准参数与 adapter owner 文档定义。

#### Scenario: 配置示例通过 schema 校验
- **WHEN** docs validator 校验 `docs/examples/json/docnav-markdown-config.json`
- **THEN** 示例符合 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** schema 约束 `defaults.pagination.limit` 为正整数、`defaults.output` 为 core document output mode，并把 `options` 作为 raw native option map
- **THEN** Markdown adapter 在消费时校验 `options.max_heading_level` 的 type 和 `1..6` 语义范围

#### Scenario: schema 不改变 core runtime 行为
- **WHEN** core CLI 读取 `docnav` core config
- **THEN** runtime 不要求加载 `docs/schemas/docnav-markdown-config.schema.json`
- **THEN** 配置读取和 source merge 由 standard parameter pipeline 与 source-level native option registry 负责
- **THEN** Markdown option 语义和 range validation 由 Markdown adapter owner 负责

### Requirement: Markdown linked adapter consumes core pagination defaults
Core document operations MUST consume `defaults.pagination.enabled`, `defaults.pagination.limit`, `--pagination enabled|disabled`, and `--limit <n>` through the shared standard-parameter source model before dispatching the linked Markdown adapter. Markdown-specific code MUST keep ownership of Markdown native options and Markdown's adapter-specific interpretation of `limit`.

#### Scenario: Core resolves pagination before Markdown dispatch
- **WHEN** core CLI runs a paginated Markdown document operation
- **THEN** standard parameter handling resolves the final limit and page before Markdown operation logic runs
- **THEN** Markdown-specific code keeps ownership of Markdown native options

#### Scenario: Markdown native option config example uses pagination limit
- **WHEN** core config schema or example for Markdown native options documents pagination defaults
- **THEN** it uses `defaults.pagination.enabled` for the default pagination state
- **THEN** it uses `defaults.pagination.limit` for the numeric budget default
- **THEN** it does not describe that budget as a core or standard-parameter unit
- **THEN** any Markdown-specific unit description remains owned by the Markdown adapter documentation
