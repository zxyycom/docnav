**一句话核心：本 delta 将 `docnav-markdown` 的 CLI 验证目标从精确 warning token 兼容改为 `clap` help 可用、宽松 argv 成功路径和必要失败边界。当前 change 只在 `openspec/changes/adopt-clap-cli-parsing/` 下形成未审核临时文档，不影响现有其它文档或主规范。**

## MODIFIED Requirements

### Requirement: Markdown adapter 必须有完整黑盒 CLI smoke 测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI smoke 测试。测试 MUST 启动构建后的 adapter binary，而不是直接调用 adapter 内部 API。测试 fixture MUST 作为项目文件固定放在指定测试目录中，MUST NOT 在测试运行时临时生成核心案例文件。smoke corpus MUST 覆盖 normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 扩展名和 `.markdown` 扩展名。smoke 测试 MUST 覆盖 `outline -> ref -> read`、`find`、`info`、`probe`、`manifest`、有效 `invoke`、CLI help 和宽松 argv 成功路径，并 MUST 覆盖 `text`、`readable-json`、`protocol-json` 输出。

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
- **THEN** 成功路径 stderr 为空或只包含非阻断诊断

#### Scenario: CLI help 可用于纠错
- **WHEN** smoke 测试执行 `docnav-markdown --help`
- **THEN** stdout 或 stderr 包含可用命令、关键参数和输出模式信息
- **THEN** 该命令不执行文档导航业务

#### Scenario: 宽松 argv 成功路径被覆盖
- **WHEN** smoke 测试执行 `docnav-markdown outline <path> --unknown extra --output readable-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令成功返回 outline readable JSON
- **THEN** 输出可以包含 warning 或诊断，但测试不要求稳定 ignored token shape

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

### Requirement: Markdown adapter 必须有负向 CLI 矩阵测试
`docnav-markdown` MUST 提供由 Node.js runner 执行的黑盒 CLI 矩阵测试，覆盖非法命令行输入、宽松 argv 输入和非法 invoke 输入。矩阵 MUST 覆盖缺 path、缺 `--ref`、缺 `--query`、unknown flag、多余 positional、当前 operation 不使用的参数、`page` 或 `limit_chars` 为 0、`page` 或 `limit_chars` 非数字、`max_heading_level` 越界、missing file、invalid ref、non-UTF-8 document、malformed invoke JSON。每个用例 MUST 按所属输出层断言 stdout、stderr 和 process exit code。

#### Scenario: 参数校验失败保持 CLI 诊断
- **WHEN** 负向矩阵执行缺 path、缺 `--ref`、缺 `--query`、非法 page、非法 limit 或非法 max heading level
- **THEN** 进程非零退出
- **THEN** stderr 包含简洁诊断
- **THEN** stdout 不包含 protocol payload 或 readable result payload

#### Scenario: unknown argv 不阻断成功路径
- **WHEN** CLI 矩阵执行 unknown flag、多余 positional 或当前 operation 不使用的参数
- **AND** 当前 operation 的必需语义参数仍可被解析
- **THEN** 进程成功退出
- **THEN** stdout 包含所选输出模式的正常结果
- **THEN** warning 或诊断可以存在，但测试不要求具体 `ignored_tokens`、kind、reason 或 token 消费顺序

#### Scenario: protocol-shaped stdout 不承载 warning
- **WHEN** CLI 矩阵以 `protocol-json`、manifest 或 probe 输出模式执行宽松 argv 成功路径
- **THEN** stdout 通过对应 protocol、manifest 或 probe schema
- **THEN** stdout 不因为 CLI warning 增加 `warnings` 字段
- **THEN** warning 或诊断只允许出现在 stderr 或非 schema stdout 之外的通道

#### Scenario: 已知使用参数仍严格校验
- **WHEN** 负向矩阵执行 `docnav-markdown outline <path> --page 0`
- **OR** 执行 `docnav-markdown outline <path> --limit-chars nope`
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
