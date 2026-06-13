本 change 的目标是用仓库内版本化 renderer config 驱动的 `readable-view` 替代 document operation 的 `text` 输出模式；本文是未审核临时 delta spec，只存在于 `openspec/changes/replace-text-with-readable-view/`，不改变现有主规范或其它文档。

## MODIFIED Requirements

### Requirement: Markdown adapter 必须有完整黑盒 CLI smoke 测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI smoke 测试。测试必须启动构建后的 adapter binary，不得直接调用 adapter 内部 API。核心 fixtures 必须是提交到项目中的固定文件，不得在测试运行时临时生成。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、`probe`、`manifest`、有效 `invoke`、CLI help、direct CLI/invoke 共享语义归一和宽松 argv 成功路径。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Warning 行为：readable warning 使用稳定 envelope；CLI argv warning 使用 `id: "cli_argv_ignored"`；测试不断言 exact token 分组、`reason` 文案或 token 消费顺序。
- Readable-view framing：版本行、合法 JSON header、静态 `/content` block 引用、UTF-8 byte length、block 起止行和正文原值还原。

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

#### Scenario: readable-view 输出 smoke 保留完整字段和 Markdown block
- **WHEN** smoke 测试执行 `outline`、`read`、`find` 和 `info` 的默认或显式 `readable-view` 输出
- **THEN** 每个 stdout 以 `@docnav-readable-view/1` 开始
- **THEN** JSON header 包含对应 operation 的全部 readable 字段和 page 状态
- **THEN** read header 的 content 原位置包含 `/content` block 引用和 UTF-8 byte length
- **THEN** `/content` block 还原值等于 readable-json content
- **THEN** stdout 不包含完整 protocol envelope
- **THEN** 成功路径 stderr 为空或只包含非阻断诊断

#### Scenario: CLI help 可用于纠错
- **WHEN** smoke 测试执行 `docnav-markdown --help`
- **OR** 执行 `docnav-markdown outline --help`
- **THEN** stdout 或 stderr 包含可用命令、关键参数、默认值或输出模式信息
- **THEN** help 只把 readable-view、readable-json 和 protocol-json 列为 document operation 输出模式
- **THEN** 该命令不执行文档导航业务

#### Scenario: document text 输出值被拒绝
- **WHEN** smoke 测试执行 `docnav-markdown outline <path> --output text`
- **THEN** 命令非零退出并报告非法 output value
- **THEN** stdout 不包含成功 readable 或 protocol payload

#### Scenario: 非文档输出通道保持独立
- **WHEN** smoke 测试执行 `docnav-markdown manifest`、`docnav-markdown probe <path>` 或 `docnav-markdown --help`
- **THEN** manifest 和 probe 按对应结构化通道输出
- **THEN** help 可以输出普通纯文本
- **THEN** 这些通道不重新暴露 document `--output text`

#### Scenario: 宽松 argv 成功路径被覆盖
- **WHEN** smoke 测试执行 `docnav-markdown outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav-markdown outline --unknown <path> --output readable-view`
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
