本 spec delta 定义 `adopt-strict-input-boundaries` 对 `markdown-navigation` 的目标变更：让 Markdown adapter 的 direct CLI、配置、adapter-owned options 和测试契约遵循 strict public input boundaries。

## MODIFIED Requirements

### Requirement: Markdown adapter 必须有完整黑盒 CLI smoke 测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI smoke 测试。测试必须启动构建后的 adapter binary，并通过真实进程边界传入 argv、stdin、cwd 和环境。核心 fixtures 必须是提交到项目中的固定文件。

Smoke suite 必须覆盖：

- Fixture corpus：normal Markdown、重复 heading、frontmatter、代码围栏伪 heading、深层 heading、无 heading、Unicode 内容、大分页内容、非 UTF-8 输入、UTF-8 BOM、CRLF 行尾、`.MD` 和 `.markdown`。
- Operations 和入口：`outline -> ref -> read`、`find`、`info`、`probe`、`manifest`、有效 `invoke`、CLI help、direct CLI/invoke 共享语义归一，以及 strict argv failure 路径。
- 输出模式：`readable-view`、`readable-json` 和 `protocol-json`。
- Strict input 行为：unknown argv、多余 positional、operation-inapplicable 参数和 undeclared native options 返回 primary `DiagnosticRecord` 投影。
- Readable-view framing：合法 JSON header、静态 `/content` block 引用、UTF-8 byte length、block 起止行和正文原值还原。

#### Scenario: Strict argv failure 被覆盖
- **WHEN** smoke 测试执行 `docnav-markdown outline <path> --unknown extra --output readable-json`
- **OR** 执行 `docnav-markdown outline --unknown <path> --output readable-view`
- **OR** 执行 `docnav-markdown outline <path> --unknown --output protocol-json`
- **AND** `<path>` 指向有效 Markdown fixture
- **THEN** 命令返回 strict input failure
- **THEN** adapter operation handler 不执行
- **THEN** failure output 投影 primary `DiagnosticRecord`

#### Scenario: 成功 smoke 使用成功 payload shape
- **WHEN** smoke suite 检查有效 `readable-json`、`readable-view` 和 `protocol-json` 输出
- **THEN** successful document output 使用 owning success payload shape
- **THEN** readable success output 使用 current success schema fields
- **THEN** protocol JSON 输出符合 protocol response envelope 结构

### Requirement: Markdown adapter 必须有负向 CLI 矩阵测试
`docnav-markdown` MUST 提供由 Node.js 执行的黑盒 CLI 矩阵测试，覆盖非法命令行输入、strict argv failure、非法配置输入和非法 invoke 输入。每个用例必须按所属输出层断言 stdout、stderr 和 process exit code。

矩阵必须覆盖：

- 必需语义：缺 path、缺 `--ref`、缺 `--query`。
- Strict argv：unknown flag、多余 positional、当前 operation 不使用的参数，包括值非法但未被当前 operation 使用的 known 参数。
- 实际使用参数失败：`page` 或 `limit_chars` 为 0、`page` 或 `limit_chars` 非数字、`output` 非法、`max_heading_level` 越界。
- 配置输入失败：显式 config override 缺失、present config invalid JSON、non-object root、未知顶层字段、未知 `defaults` 字段和 undeclared `options` key。
- 业务和输入错误：missing file、invalid ref、non-UTF-8 document。
- Invoke 输入错误：malformed invoke JSON 作为 transport decode failure；unknown envelope fields 停在 protocol direct input boundary；known argument 类型错误或 unmapped arguments 由标准参数/typed-field processing 产生 protocol-shaped failure。

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
- **WHEN** CLI 矩阵以 `--output protocol-json` 执行 strict argv failure、config input failure 或 invoke validation failure
- **THEN** stdout 通过 protocol response schema
- **THEN** failure envelope 投影一个 primary `DiagnosticRecord`
- **THEN** adapter operation handler 不执行

### Requirement: docnav-markdown direct CLI 支持 JSON 配置文件
`docnav-markdown` direct CLI MUST 读取项目级 `.docnav/docnav-markdown.json` 和默认用户配置目录下的 `docnav-markdown.json` 配置，并 MUST 支持 SDK-owned `--project-config-path <path>` 和 `--user-config-path <path>` 覆盖这两个配置文件路径；默认用户配置目录未提供时使用当前调用位置（启动 cwd）。首期配置 MUST 支持 `defaults.limit_chars`、`defaults.output` 和 `options.max_heading_level`。Document operation help MUST 展示两个配置路径参数。

默认配置路径缺失表示该配置 source absent。显式覆盖路径缺失、不可读、不是文件、invalid JSON 或 non-object root MUST fail。默认配置路径一旦存在但不可读、invalid JSON 或 non-object root MUST fail。未知顶层字段、未知 `defaults` 字段和 undeclared `options` key MUST fail。Declared `options` keys 在 handler execution 前进入 adapter-owned native option validation。

#### Scenario: 默认配置路径缺失表示 absence
- **WHEN** 项目级或用户级默认配置路径不存在
- **AND** 调用方没有显式覆盖该层配置路径
- **THEN** 该配置 source absent
- **THEN** Markdown adapter 使用其它有效来源和内置默认值继续构造参数来源

#### Scenario: 显式配置路径不可用时失败
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path missing.json`
- **THEN** CLI 返回配置输入错误
- **THEN** failure diagnostic 包含 config source location 和 repair guidance
- **THEN** operation handler 不执行

#### Scenario: present config invalid 时失败
- **WHEN** 默认或显式配置路径存在
- **AND** 配置内容 unreadable、invalid JSON、non-object root、包含未知顶层字段或未知 `defaults` 字段
- **THEN** CLI 返回配置输入错误
- **THEN** 已知字段不被用于继续成功路径

#### Scenario: options key 由 adapter owner 校验
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** SDK 将 `options` object 合并为 adapter-owned native options 参数来源
- **THEN** Markdown native option owner 校验 `max_heading_level`
- **WHEN** 配置文件包含 undeclared `options` key
- **THEN** CLI 返回 native option input diagnostic

### Requirement: docnav-markdown 配置必须由 smoke 和矩阵测试覆盖
`docnav-markdown` black-box CLI smoke 和矩阵 MUST 覆盖配置文件读取、优先级、配置 source absence、配置 input failure、help 参数展示，以及 `invoke` request `arguments` 与配置/default sources 进入同一标准参数解析的边界。

#### Scenario: 配置 source absence 被覆盖
- **WHEN** 默认项目级或用户级配置文件不存在
- **THEN** smoke 测试证明该 source absent
- **THEN** operation 可以使用其它有效来源和内置默认值成功

#### Scenario: 配置 input failure 被覆盖
- **WHEN** 显式 config override 缺失、present config invalid JSON、non-object root、未知字段或 undeclared option key 出现
- **THEN** 矩阵测试证明 CLI 返回 strict failure
- **THEN** failure output 投影一个 primary `DiagnosticRecord`

#### Scenario: Valid options 仍影响 Markdown 导航语义
- **WHEN** 项目级 `docnav-markdown.json` 设置 `options.max_heading_level`
- **AND** 该 key 由 Markdown native option owner 声明
- **THEN** 测试证明 `outline` 和 `find` 都消费适用的 `options.max_heading_level`
- **THEN** direct CLI 和有效 `invoke` 入口通过同一标准参数解析边界进入 handler
