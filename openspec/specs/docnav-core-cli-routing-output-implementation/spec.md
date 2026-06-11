# docnav-core-cli-routing-output-implementation Specification

## Purpose
定义 `docnav` 核心 CLI 的命令解析、adapter 选择、invoke 调用、输出分层、配置和稳定错误映射行为。
## Requirements
### Requirement: 核心 CLI 必须作为独立 docnav 可执行入口交付
`docnav` MUST 作为独立 Rust workspace crate 和可执行二进制交付。格式解析 MUST 由选中的 adapter 完成，核心 CLI MUST 只负责命令解析、adapter 调用、协议校验和输出映射。

#### Scenario: 构建核心 CLI
- **WHEN** 构建 workspace 中的 `docnav` package
- **THEN** 产出名为 `docnav` 的可执行文件
- **THEN** 该可执行文件可以解析核心 CLI 命令

### Requirement: 核心 CLI 必须实现文档操作命令
`docnav` MUST 实现 `outline`、`read`、`find` 和 `info`，并 MUST 支持各命令对应的 `--adapter`、`--page`、`--limit-chars` 和 `--output` 参数约束。`info` invoke 请求 MUST 只包含 info operation 需要的 core 参数。

#### Scenario: 执行 outline 命令
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 解析最终 page 和 limit_chars
- **THEN** `docnav` 启动选中 adapter 的 invoke

#### Scenario: 执行 info 命令
- **WHEN** 调用方执行 `docnav info docs/guide.md`
- **THEN** `docnav` 启动选中 adapter 的 invoke
- **THEN** invoke 请求不包含 page 或 limit_chars

### Requirement: 核心 CLI 必须兼容未知、多余和未使用参数
`docnav` core CLI MUST 使用 `clap` 或 `clap` builder API 作为命令、子命令、固定参数、默认值、枚举值和 help 的 argv 结构解析基础。Document operation argv 必须先映射为 canonical document operation input 或等价 semantic request，再进入 adapter routing、invoke request 构造和 output dispatch。

Core CLI 容错规则如下：

- 当前 operation 的必需语义存在且实际使用参数有效时，未知 flag、多余 positional 和当前 operation 不使用的参数不得阻断成功。
- 当前 operation 实际使用的参数必须保持严格；缺值或值非法时必须返回 `INVALID_REQUEST`。
- `clap` 负责已知命令、已知参数声明、默认值、枚举和 help；Docnav 在确定 command/operation 后只对当前 operation 实际使用的参数做类型、范围和枚举校验，并受控收集 unknown、extra positional 和 unused known 参数的原始 token 生成 warning metadata，不复制业务参数解释、默认值归一或 request 构造逻辑。
- 每个被忽略的 argv family 必须形成阅读层 warning 或 stderr 诊断；输出通道按当前输出模式决定。
- Readable warning item 必须使用稳定 warning envelope：稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。
- CLI argv 兼容 warning 必须使用 `id: "cli_argv_ignored"`，并可在 `details.tokens` 中列出相关 argv token。
- Adapter candidate warning 必须保留 `id: "adapter_candidate_failure"`，并在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected` 等 candidate 字段。
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
- **THEN** stdout 包含 `warnings`
- **THEN** 每个 warning item 通过 readable schema
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

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
- **WHEN** 调用方执行 `docnav info docs/guide.md --page nope --output readable-json`
- **OR** 执行 `docnav info docs/guide.md --limit-chars nope --output readable-json`
- **AND** info 所需 path 和 output 可被解析
- **THEN** `docnav` 执行 info
- **THEN** info invoke 请求不包含 page 或 limit_chars
- **THEN** page 或 limit_chars 参数最多以原始 token 形成阅读层 warning 或 stderr 诊断
- **THEN** CLI 不因该 unused known 参数无法通过其它 operation 的类型或范围校验而失败

#### Scenario: Help 不执行业务
- **WHEN** 调用方执行 `docnav --help`
- **OR** 执行 core 子命令 help
- **THEN** 输出列出可用命令、关键参数、默认值或可选值
- **THEN** 该命令不读取文档、不选择 adapter、不启动 adapter invoke

#### Scenario: Core non-document commands 保持非文档边界
- **WHEN** 调用方执行 `docnav config get/set/unset/list`
- **OR** 执行 `docnav init`
- **OR** 执行 `docnav doctor`
- **OR** 执行 `docnav version`
- **THEN** `docnav` 通过 `clap` 或 `clap` builder 解析出类型化非文档命令
- **THEN** 这些命令不进入 canonical document operation input 或等价 semantic request
- **THEN** 这些命令不执行 adapter routing、adapter invoke 或文档导航业务
- **THEN** 代表性成功、失败、stdout、stderr 和 exit code 行为由 core CLI smoke 或等价测试覆盖

### Requirement: path 必须规范化并支持项目根外文件
`docnav` MUST 将输入 path 解析为使用 `/` 的规范路径，并 MUST 允许读取项目根外的可访问文件。

#### Scenario: 项目根外路径
- **WHEN** 调用方传入会解析到项目根外且可访问的 path
- **THEN** `docnav` 保留该规范路径
- **THEN** `docnav` 仍可继续选择 adapter 并启动 adapter 进程

#### Scenario: 不可访问路径
- **WHEN** 调用方传入不存在或不可读的 path
- **THEN** `docnav` 返回文档路径错误
- **THEN** 不启动 adapter 进程

### Requirement: 核心配置 MVP 必须有限且可审计
`docnav` MUST 在本 change 中只支持 `defaults.adapter`、`defaults.limit_chars` 和 `defaults.output` 三个核心配置 key，并 MUST 按显式参数、项目配置、用户配置、内置默认值的优先级解析最终 core 参数值。Core 参数的默认值来源 MUST 限定为 CLI、配置和内置默认值；adapter manifest 只参与 adapter 识别和当前契约校验。

#### Scenario: 配置 adapter 预选
- **WHEN** 调用方未传入 `--adapter`
- **AND** 项目配置设置了 `defaults.adapter`
- **THEN** `docnav` 使用该 adapter id 作为预选 adapter

#### Scenario: page 不可配置
- **WHEN** 调用方省略 `--page`
- **THEN** `docnav` 使用 page `1`
- **THEN** 项目配置和用户配置保持初始 page 为 `1`

#### Scenario: 未知配置 key
- **WHEN** 调用方执行 `docnav config get unknown.key`
- **THEN** `docnav` 返回 `INVALID_REQUEST`

#### Scenario: 默认写项目配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json`
- **THEN** `docnav` 写入 `<project-root>/.docnav/docnav.json`
- **THEN** 当前项目的生效配置包含 `defaults.output`

#### Scenario: 写用户配置
- **WHEN** 调用方执行 `docnav config set defaults.output readable-json --user`
- **THEN** `docnav` 写入用户配置文件
- **THEN** 未设置项目同名 key 时该用户配置成为生效值

#### Scenario: 列出当前生效配置
- **WHEN** 调用方执行 `docnav config list`
- **THEN** 输出包含所有支持 key 的当前生效值
- **THEN** 输出标明每个值来自项目配置、用户配置、内置默认值或未设置状态

#### Scenario: 按文档上下文列出最终配置
- **WHEN** 调用方执行 `docnav config list --path docs/guide.md --operation outline`
- **THEN** `docnav` 按文档命令规则解析 path 并选择 adapter
- **THEN** 输出包含选中 adapter id
- **THEN** 输出包含该文档和 operation 下的最终默认参数及其来源

### Requirement: 核心管理命令必须提供 MVP 行为
`docnav` MUST 实现 `init`、`doctor` 和 `version` 的可验证基础行为。

#### Scenario: init 幂等创建项目配置
- **WHEN** 调用方执行 `docnav init`
- **THEN** `docnav` 创建 `<project-root>/.docnav/docnav.json`
- **AND** 重复执行不会破坏已有配置

#### Scenario: version 输出 crate 版本
- **WHEN** 调用方执行 `docnav version`
- **THEN** stdout 包含 `docnav` crate version

#### Scenario: doctor 检查配置和 adapter CLI
- **WHEN** 调用方执行 `docnav doctor`
- **THEN** `docnav` 检查项目配置、用户配置和临时 adapter registry
- **THEN** `docnav` 检查已记录 adapter CLI 是否可启动并可返回 manifest
- **THEN** 输出包含 checks 数组
- **AND** 存在失败检查项时进程非零退出

### Requirement: adapter 选择必须先校验一个预选 adapter
`docnav` MUST 先确定一个预选 adapter id，并 MUST 使用统一的候选评估规则决定选中、继续遍历或直接失败。Adapter 评估 MUST 以 registry 记录解析、manifest 当前 schema、当前契约语义和 adapter probe 结果为准。

#### Scenario: 显式 adapter 记录解析失败后继续
- **WHEN** 调用方传入 `--adapter docnav-markdown` 但 registry 中无法解析该 adapter 记录
- **THEN** `docnav` 保留失败证据
- **THEN** `docnav` 生成 warning
- **THEN** `docnav` 调用 registry 遍历函数继续尝试其它 adapter

#### Scenario: probe 有效不支持后继续
- **WHEN** 候选 adapter probe 返回符合当前 schema 和语义的 `supported: false`
- **THEN** `docnav` 保留 `PROBE_UNSUPPORTED` 候选证据
- **THEN** `docnav` 继续 registry 遍历

#### Scenario: 未显式指定时先 core 推断
- **WHEN** 调用方没有传入 `--adapter`
- **AND** 配置没有指定 `defaults.adapter`
- **THEN** `docnav` 使用候选 manifest 的格式信息推断一个预选 adapter id
- **THEN** `docnav` 先校验该预选 adapter

#### Scenario: 预选 adapter manifest 当前契约不一致后继续
- **WHEN** 预选 adapter manifest 缺少 `docnav` 当前 CLI 选择 adapter 所需字段
- **THEN** `docnav` 保留候选证据
- **THEN** `docnav` 继续 registry 遍历

#### Scenario: registry 遍历候选当前契约不一致后继续
- **WHEN** registry 遍历中的候选 adapter manifest 或 probe 输出字段缺失、类型不符或语义校验失败
- **THEN** `docnav` 保留候选证据
- **THEN** `docnav` 继续 registry 遍历
- **THEN** 若后续候选成功，输出层按模式呈现前序候选 warning

#### Scenario: 所有阶段失败
- **WHEN** 没有 adapter 能校验目标文档
- **THEN** `docnav` 返回 `FORMAT_UNKNOWN`
- **THEN** 错误 details 包含候选证据
- **THEN** 候选证据是 JSON 数组
- **THEN** 每条候选证据包含 adapter_id、stage、code、reason 和 details

### Requirement: 临时 adapter 记录必须足以启动 adapter
`docnav` MUST 在本 change 中支持项目级临时 adapter registry，文件 MUST 位于 `<project-root>/.docnav/adapters.json`，记录 MUST 至少包含 adapter id 和相对项目根的命令路径。正式黑白名单、版本选择和安装记录 MUST 由 adapter 管理 change 交付。

#### Scenario: 使用临时 adapter 记录
- **WHEN** registry 中存在 adapter id 和相对命令路径
- **THEN** `docnav` 可以按 adapter id 解析命令
- **THEN** `docnav` 使用该命令执行 `probe` 和 `invoke`

#### Scenario: registry 保留遍历顺序
- **WHEN** registry 的 `adapters` 数组包含多个候选
- **THEN** `docnav` 按数组顺序遍历候选

#### Scenario: registry 命令路径非法
- **WHEN** registry 中的 adapter command 是绝对路径或无法相对项目根解析
- **THEN** `docnav` 返回 `INVALID_REQUEST`
- **THEN** 不启动该 adapter

### Requirement: invoke 请求必须包含最终显式参数
`docnav` MUST 在启动 adapter invoke 前解析配置和默认值，并 MUST 将最终 page、limit_chars、ref 和 query 等 core 通用参数显式写入 invoke 请求。Core `docnav` MUST NOT synthesize format-specific `options`; adapter manifest MUST NOT be an options or default-parameter source.

#### Scenario: 省略 page
- **WHEN** 调用方省略 page
- **THEN** `docnav` 传给 invoke 的 page 为 `1`

#### Scenario: 不写入格式 options
- **WHEN** 选中 adapter manifest 通过当前 schema 校验
- **THEN** `docnav` 仍能解析 core 默认参数
- **THEN** invoke 请求不包含由 manifest、配置或隐式默认值合成的格式 options

#### Scenario: invoke 当前契约不一致
- **WHEN** 选中 adapter 的 invoke 输出字段缺失、类型不符、operation/result shape 不匹配或语义校验失败
- **THEN** `docnav` 返回 adapter/protocol 错误
- **THEN** `docnav` 不继续尝试其它 adapter

### Requirement: 输出模式必须按协议层和阅读层分离
`docnav --output protocol-json` MUST 输出原始 protocol response envelope，且不得增加 CLI warning 字段。默认 text 和 readable-json 输出必须保持为阅读层结果。readable-json 在存在 ignored argv 或 adapter candidate warning 时必须包含顶层 `warnings`。每个 warning item 必须使用稳定 warning envelope，包含 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。CLI argv warning 必须使用 `id: "cli_argv_ignored"`，相关 argv token 只能作为 `details.tokens` 等 family-specific detail 表达。CLI argv exact token 分组、`reason` 文案和消费顺序不稳定。

#### Scenario: readable-json outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md --output readable-json`
- **THEN** 输出包含 entries、page 等 outline readable fields
- **THEN** 输出不包含 protocol envelope 字段

#### Scenario: readable-json warning envelope
- **WHEN** 调用方执行带有未知参数但其它参数有效的 readable-json 命令
- **THEN** 输出包含该 operation 的 readable 字段
- **THEN** 输出包含 `warnings` 数组
- **THEN** 每个 warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

#### Scenario: protocol-json warning
- **WHEN** 调用方执行带有未知参数但其它参数有效的 protocol-json 命令
- **THEN** 输出包含完整 protocol response envelope
- **THEN** 输出不包含 `warnings` 数组
- **THEN** stderr 包含 warning 诊断

### Requirement: 核心 CLI 必须保留 adapter 业务语义
`docnav` MUST 在输出映射中保留 adapter 返回的 ref、display、content、content_type、cost 和 page。

#### Scenario: read 保留 content_type
- **WHEN** adapter read 返回 `content_type: "text/markdown"`
- **THEN** `docnav` readable-json read 输出包含相同 content_type

### Requirement: 核心 CLI 必须实现稳定错误和退出码映射
`docnav` MUST 将输入错误、文档/ref/格式错误、协议或 adapter 进程错误、内部错误映射到主规范定义的退出码，并 MUST 保持错误 code 稳定。

#### Scenario: ref 不存在
- **WHEN** adapter 返回 `REF_NOT_FOUND`
- **THEN** `docnav` 保留该错误 code
- **THEN** CLI 退出码为文档/ref/格式错误对应值

#### Scenario: config key 不存在
- **WHEN** 调用方执行 `docnav config get <key>` 且 key 不存在
- **THEN** `docnav` 返回 `INVALID_REQUEST`

#### Scenario: adapter 响应无法校验
- **WHEN** 选中 adapter 的 invoke stdout 不是单一 protocol JSON 响应
- **THEN** `docnav` 返回协议或 adapter 进程错误
- **THEN** CLI 退出码为协议或 adapter 进程错误对应值

