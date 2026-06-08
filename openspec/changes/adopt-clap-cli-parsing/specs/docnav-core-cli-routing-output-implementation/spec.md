范围说明：本 delta 只调整核心 `docnav` CLI argv 解析、宽松 CLI 成功路径、readable warning shape 和输出通道边界。不改变文档操作语义或 protocol response 字段。

## MODIFIED Requirements

### Requirement: 核心 CLI 必须兼容未知、多余和未使用参数
`docnav` core CLI MUST 使用 `clap` 或 `clap` builder API 作为命令、子命令、固定参数、默认值、枚举值和 help 的解析基础。Document operation argv 必须先映射为 canonical document operation input 或等价 semantic request，再进入 adapter routing、invoke request 构造和 output dispatch。

Core CLI 容错规则如下：

- 当前 operation 的必需语义存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数不得阻断成功。
- 当前 operation 实际使用的参数必须保持严格；缺值或值非法时必须返回 `INVALID_REQUEST`。
- Readable warning item 必须使用稳定 warning envelope：稳定 `kind`、非空 `reason`、`ignored_tokens: string[]` 和可选 family-specific 字段。
- CLI argv 兼容 warning 必须使用 `kind: "cli_argv_ignored"`。
- Adapter candidate warning 必须保留 `kind: "adapter_candidate_failure"`，并保留 `adapter_id`、`stage`、`code` 等 candidate 字段。
- CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不得作为稳定契约。
- `protocol-json` stdout 不得给 protocol response envelope 增加 warning 字段。

#### Scenario: 核心 CLI 进入共享 semantic request 管道
- **WHEN** 调用方执行有效的 `docnav outline/read/find/info` CLI 命令
- **THEN** `clap` 或 `clap` builder 解析出类型化命令
- **THEN** document CLI input 映射为 canonical document operation input 或等价 semantic request
- **THEN** adapter routing、invoke request 构造和 output mode 分流使用共享逻辑
- **THEN** CLI 不创建独立业务参数解释路径

#### Scenario: 未知 argv 不阻断有效文档操作
- **WHEN** 调用方执行 `docnav outline docs/guide.md --future extra --output readable-json`
- **AND** path 和 output 可被解析
- **THEN** `docnav` 继续选择 adapter 并执行 outline
- **THEN** stdout 包含 outline readable JSON
- **THEN** 若 stdout 包含 `warnings`，每个 warning item 通过 readable schema
- **THEN** CLI argv warning 使用 `kind: "cli_argv_ignored"`
- **THEN** 测试不要求 exact ignored-token 分组、`reason` 文案或 token 消费顺序

#### Scenario: 未知 argv 不吞已知输出模式
- **WHEN** 调用方执行 `docnav outline docs/guide.md --future --output protocol-json`
- **THEN** `docnav` 仍解析并使用 `protocol-json` 输出模式
- **THEN** stdout 是通过 protocol response schema 的 envelope
- **THEN** stdout 不包含 `warnings`
- **THEN** warning 或诊断只写入 stderr

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** 调用方执行 `docnav outline docs/guide.md --page 0`
- **OR** 执行 `docnav outline docs/guide.md --limit-chars nope`
- **OR** 执行 `docnav outline docs/guide.md --output nope`
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** CLI 不通过宽松 argv 策略忽略该参数并继续

#### Scenario: 当前 operation 不使用的参数宽松
- **WHEN** 调用方执行 `docnav info docs/guide.md --page 9 --output readable-json`
- **AND** info 所需 path 和 output 可被解析
- **THEN** `docnav` 执行 info
- **THEN** info invoke 请求不包含 page 或 limit_chars
- **THEN** page 参数最多形成阅读层 warning 或 stderr 诊断

#### Scenario: Help 不执行业务
- **WHEN** 调用方执行 `docnav --help`
- **OR** 执行 core 子命令 help
- **THEN** 输出列出可用命令、关键参数、默认值或可选值
- **THEN** 该命令不读取文档、不选择 adapter、不启动 adapter invoke

### Requirement: 输出模式必须按协议层和阅读层分离
`docnav --output protocol-json` MUST 输出原始 protocol response envelope，且不得增加 CLI warning 字段。默认 text 和 readable-json 输出必须保持为阅读层结果。readable-json 可以在 readable schema 允许时包含顶层 `warnings`。每个 warning item 必须使用稳定 warning envelope，包含 `kind`、非空 `reason`、`ignored_tokens: string[]` 和可选 family-specific 字段。CLI argv warning 必须使用 `kind: "cli_argv_ignored"`。CLI argv exact token 分组、`reason` 文案和消费顺序不稳定。

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出包含 entries、page 等 outline readable fields
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: readable-json warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的 readable-json 命令
- **THEN** 输出包含该 operation 的 readable 字段
- **THEN** 输出可以包含 `warnings` 数组
- **THEN** 每个 warning item 包含稳定 `kind`、非空 `reason`、`ignored_tokens` 数组和可选 family-specific 字段
- **THEN** CLI argv warning 使用 `kind: "cli_argv_ignored"`
- **THEN** 测试不要求 exact ignored-token 分组、`reason` 文案或 token 消费顺序

#### Scenario: protocol-json warning
- **WHEN** 调用方执行带有未知参数但其它参数有效的 protocol-json 命令
- **THEN** 输出包含完整 protocol response envelope
- **THEN** 输出不包含 `warnings` 数组
- **THEN** stderr 可以包含 warning 诊断
