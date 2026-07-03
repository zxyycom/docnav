本 spec delta 定义 `adopt-strict-input-boundaries` 对 `markdown-navigation` 的目标变更：让 core-linked Markdown adapter、core 配置来源、adapter-owned options 和 core CLI 测试契约遵循 strict public input boundaries。

## MODIFIED Requirements

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
