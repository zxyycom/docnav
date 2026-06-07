# markdown-adapter-v0-implementation Specification

## Purpose
Define the implemented Markdown v0 adapter behavior for manifest, probe, outline, read, find, info, ref handling, pagination, direct CLI output, and required boundary-case verification.
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
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI smoke 测试。测试 MUST 启动构建后的 adapter binary，而不是直接调用 adapter 内部 API。测试 fixture MUST 作为项目文件固定放在指定测试目录中，MUST NOT 在测试运行时临时生成核心案例文件。smoke corpus MUST 覆盖 normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 扩展名和 `.markdown` 扩展名。smoke 测试 MUST 覆盖 `outline -> ref -> read`、`find`、`info`、`probe`、`manifest` 和有效 `invoke`，并 MUST 覆盖 `text`、`readable-json`、`protocol-json` 输出。

#### Scenario: Node.js runner 使用构建产物
- **WHEN** smoke 测试运行
- **THEN** 测试先使用已构建的 `docnav-markdown` binary 路径启动真实进程
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

#### Scenario: text 输出 smoke 保留关键可读信息
- **WHEN** smoke 测试执行 `outline`、`read`、`find` 和 `info` 的 `text` 输出
- **THEN** stdout 包含对应 operation 的关键可读信息，例如 ref、content、content_type、cost、capabilities 或 page 状态
- **THEN** stdout 不包含完整 protocol envelope JSON
- **THEN** 成功路径 stderr 为空

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
- **THEN** manifest 声明 Markdown capabilities 和 recommended parameters
- **THEN** probe 返回格式证据且不包含 outline/read payload
- **THEN** find 返回带 ref 和 page 状态的 matches
- **THEN** info 返回 Markdown 摘要和 capabilities
- **THEN** 有效 invoke 请求从 stdin 输入后在 stdout 返回成功 protocol envelope

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
`docnav-markdown` MUST 提供由 Node.js runner 执行的黑盒 CLI 矩阵测试，覆盖非法命令行输入、兼容性 warning 输入和非法 invoke 输入。矩阵 MUST 覆盖缺 path、缺 `--ref`、缺 `--query`、unknown flag、多余 positional、当前 operation 不使用的已知 flag、`page` 或 `limit_chars` 为 0、`page` 或 `limit_chars` 非数字、`max_heading_level` 越界、missing file、invalid ref、non-UTF-8 document、malformed invoke JSON。每个用例 MUST 按所属输出层断言 stdout、stderr 和 process exit code。

#### Scenario: 参数校验失败保持 CLI 诊断
- **WHEN** 负向矩阵执行缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit 或非法 max heading level
- **THEN** 进程非零退出
- **THEN** stderr 包含简洁诊断
- **THEN** stdout 不包含 protocol payload 或 readable result payload

#### Scenario: 兼容参数 warning 后继续
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的已知 flag
- **THEN** warning 包含具体 `ignored_tokens`、kind 和 reason
- **THEN** 若其它参数有效，进程成功退出
- **THEN** stdout 包含所选输出模式的正常结果
- **THEN** text 输出在正常结果后拼接 warning 文本
- **THEN** readable-json 输出包含 `warnings` 数组
- **THEN** protocol-json、manifest 和 probe stdout 不包含 `warnings` 字段，warning 写入 stderr

#### Scenario: unknown flag 的后续普通 token 有明确归属
- **WHEN** CLI 矩阵执行 `docnav-markdown outline <path> --future extra`
- **THEN** `--future` 被归属为 unknown flag warning 的 ignored token
- **THEN** `extra` 继续按普通 token 处理
- **THEN** 因 outline 已接收 `<path>` positional，`extra` 被归属为多余 positional warning 的独立 ignored token

#### Scenario: unknown flag 不吞后续已知 flag
- **WHEN** CLI 矩阵执行 `docnav-markdown outline <path> --future --output protocol-json`
- **THEN** `--future` 被 warning 后忽略
- **THEN** `--output protocol-json` 仍生效
- **THEN** stdout 是通过 protocol response schema 的 protocol JSON 且不包含 `warnings`
- **THEN** stderr 包含 `--future` warning

#### Scenario: 已知 flag 的值紧跟解析
- **WHEN** 负向矩阵执行 `docnav-markdown read <path> --ref --future-value`
- **THEN** `--future-value` 作为 `--ref` 的值传入
- **THEN** 命令按该 ref 执行业务逻辑并返回对应 operation 结果或稳定 ref 错误
- **THEN** stderr 不包含缺少 `--ref` 值的诊断

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

### Requirement: Markdown heading ref 必须使用 canonical line-ordinal-path 格式
`docnav-markdown` MUST 在 heading path occurrence ordinal 为 `1` 时生成 `L{line}:{path}`，并在 occurrence ordinal 大于 `1` 时生成 `L{line}#{ordinal}:{path}`。`path` MUST 表示 heading breadcrumb，不是文件路径。`docnav-markdown` MUST 继续生成并接受 `doc:full` 作为全文 fallback ref；`doc:full` 不属于 heading ref 格式。

#### Scenario: 无重复 heading 时省略默认 ordinal
- **WHEN** Markdown 文档包含唯一 heading path，例如 `Guide` 和 `Guide > Install`
- **THEN** outline 输出 `L1:Guide` 和 `L5:Guide > Install` 形式的 ref
- **THEN** outline MUST NOT 为这些首个 occurrence 输出 `#1`

#### Scenario: 重复完整 heading path 时输出重复 ordinal
- **WHEN** Markdown 文档包含重复完整 heading path，例如 `Repeat` 和 `Repeat > Child`
- **THEN** outline 为首个 occurrence 输出 `L1:Repeat` 和 `L5:Repeat > Child` 形式的 ref
- **THEN** outline 为后续 occurrence 输出 `L9#2:Repeat` 和 `L13#2:Repeat > Child` 形式的 ref

#### Scenario: read 接受 canonical heading ref
- **WHEN** 调用方把 canonical heading ref，例如 `L5:Guide > Install` 或 `L9#2:Repeat`，传给 read
- **THEN** read 返回唯一匹配的 Markdown section
- **THEN** content_type 为 `text/markdown`

#### Scenario: read 接受显式 default ordinal
- **WHEN** 调用方把显式 default ordinal ref，例如 `L1#1:Guide`，传给 read
- **THEN** read 定位该 heading path 的首个 occurrence
- **THEN** 生成器仍省略 `#1`

#### Scenario: read 拒绝 legacy bracketed ordinal suffix
- **WHEN** 调用方把使用旧方括号 ordinal 后缀的 heading ref 传给 read
- **THEN** read 返回现有稳定 ref 错误
- **THEN** read MUST NOT 把该旧 ref 解析到 Markdown section
