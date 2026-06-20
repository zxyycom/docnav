# docnav-contracts Specification

## Purpose
定义 Docnav v0 长期文档契约，约束原始协议、阅读输出、职责边界、阅读路径、示例材料和自动化验证映射。
## Requirements
### Requirement: 原始协议与阅读输出分层
项目 MUST 将 adapter invoke 原始协议与 CLI/MCP 阅读输出定义为两个独立语义层。原始协议层 MUST 作为机器稳定接口；阅读输出层 MUST 优先服务 AI 和人类阅读，不作为长期机器解析接口。两层 MUST 复用业务语义，并以各自 schema 校验字段形状。

#### Scenario: 原始协议响应自描述 operation
- **WHEN** adapter invoke 返回成功响应
- **THEN** 响应 envelope 包含与请求一致的 `operation`
- **THEN** protocol response schema 使用 `operation` 校验对应 result 类型

#### Scenario: MCP 返回阅读结果
- **WHEN** `docnav-mcp` 收到 `docnav` readable 结果
- **THEN** MCP structuredContent 使用精简 readable schema
- **THEN** MCP 输出不包含 `protocol_version`、`request_id`、`operation` 或 `ok`
- **THEN** MCP structuredContent 不替代完整 protocol envelope

#### Scenario: Read 保留内容类型
- **WHEN** adapter read 返回内容
- **THEN** protocol result、readable JSON 和 MCP structuredContent 均包含 `content_type`
- **THEN** `content_type` 不参与字符预算裁剪

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST 负责项目根解析、核心配置、adapter 发现、安装、更新、移除、选择、invoke 启动、协议字段校验、输出模式和错误映射。

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 根据 path、配置、manifest、扩展名和 probe 选择 adapter
- **THEN** `docnav` 将 page 和 limit_chars 等 core 通用参数写入显式 invoke 请求
- **THEN** `docnav` 不从 manifest、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: 正式安装 adapter
- **WHEN** 调用方执行 `docnav adapter install <source>`
- **THEN** `docnav` 只接受 GitHub 链接或本地可执行文件来源
- **THEN** `docnav` 解析安装来源并执行 adapter `manifest`
- **THEN** manifest 只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities
- **THEN** manifest 通过当前 schema、必需字段、字段类型和语义校验后才注册 adapter
- **THEN** 本地可执行文件来源必须记录 SHA-256 hash

#### Scenario: 正式更新 adapter
- **WHEN** 调用方执行 `docnav adapter update <adapter-id>`
- **THEN** `docnav` 使用已记录来源获取候选新制品
- **THEN** 新制品 manifest 通过 schema、必需字段、字段类型和语义校验后才替换旧记录
- **THEN** 校验失败时保留旧记录并返回结构化错误

#### Scenario: 本地 exe hash 失配
- **WHEN** 已注册的本地可执行文件 hash 与安装记录不一致
- **THEN** `docnav` 在 invoke 前阻断该 adapter
- **THEN** `docnav` 返回 `ADAPTER_UNAVAILABLE` 且 `details.reason` 为 `hash_mismatch`

### Requirement: Adapter 选择按阶段校验
`docnav` MUST 先按显式格式或 content type 提示校验 adapter；失败后 MUST 按扩展名匹配候选 adapter 并校验；仍失败时 MUST 逐个 probe 已安装 adapter，直到成功或全部失败。`docnav` MUST NOT 只凭格式提示或扩展名静默选择 adapter。

#### Scenario: 显式格式优先
- **WHEN** 调用方提供 `--format markdown` 或等价 MCP `format`
- **THEN** `docnav` 优先使用 manifest 中匹配 `formats[].id` 的 adapter 候选并执行校验
- **THEN** 校验失败时继续扩展名匹配阶段

#### Scenario: 显式 content type 优先
- **WHEN** 调用方提供 `--format text/markdown` 或等价 MCP `format`
- **THEN** `docnav` 优先使用 manifest 中匹配 `formats[].content_types[]` 的 adapter 候选并执行校验
- **THEN** 校验成功时选中该 adapter

#### Scenario: 扩展名和全量 probe
- **WHEN** 显式格式阶段未选中 adapter
- **THEN** `docnav` 按 manifest `formats[].extensions[]` 匹配 path 扩展名并校验候选 adapter
- **THEN** 扩展名候选都失败时，`docnav` 逐个 probe 已安装 adapter
- **THEN** 全部阶段失败时返回 `FORMAT_UNKNOWN`

### Requirement: MCP 是接入层
`docnav-mcp` MUST 是 Node.js / JavaScript MCP bridge，MUST 通过 stdio 提供 MCP transport，MUST 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`，并 MUST 将 tool call 直接映射为核心 `docnav` CLI 调用。

#### Scenario: MCP 调用 read
- **WHEN** AI Client 调用 `document_read`
- **THEN** `docnav-mcp` 将参数映射到核心 `docnav` CLI
- **THEN** `docnav-mcp` 将 `docnav` 结果转换为 TextContent 和 structuredContent
- **THEN** `docnav-mcp` 使用内联或随包打包的 tool `outputSchema`
- **THEN** adapter 路由和下级适配层调用只由 `docnav` 完成

#### Scenario: MCP 传递格式提示
- **WHEN** AI Client 在 tool call 中提供 `format`
- **THEN** `docnav-mcp` 将其映射为 `docnav --format`
- **THEN** `docnav-mcp` 不自行解析该格式提示

### Requirement: 接入方式共享 `docnav` 契约
直接 CLI、skill、AGENTS.md / system prompt 和 MCP MUST 共享 `docnav` 的 path、ref、page、limit_chars、输出模式和错误契约。

#### Scenario: Agent 通过项目规则读取文档
- **WHEN** 项目规则提示 agent 读取大型文档
- **THEN** agent 使用 `docnav outline/read/find/info`
- **THEN** agent 将 outline 返回的 ref 原样传入 read

### Requirement: Markdown v0 实现全部首期能力
Markdown v0 adapter MUST 实现 `outline`、`read`、`find` 和 `info` 全部能力；`outline -> ref -> read` MUST 作为首要纵向阅读链路。

#### Scenario: Markdown capability 集合
- **WHEN** 调用方读取 Markdown adapter manifest
- **THEN** `capabilities` 包含 `outline`、`read`、`find` 和 `info`
- **THEN** 每个能力都有对应 CLI、invoke 和 MCP 映射验证

### Requirement: Outline 只使用扁平条目
共享协议 MUST 将 outline 定义为扁平 entries；每条 entry MUST 包含 `ref` 和 `display`。

#### Scenario: 嵌套 Markdown heading
- **WHEN** Markdown 文档包含嵌套 heading
- **THEN** outline 按文档顺序返回扁平条目
- **THEN** 层级关系由适配器生成的 ref 或 display 表达

### Requirement: 默认结果有限且可继续
每个导航操作 MUST 有明确字符预算默认值；分页操作 MUST 使用正整数 page 和 `limit_chars`，并返回请求 page 加 1 或 null。
`limit_chars` MUST 按 UTF-8 解码后的 Unicode 字符计数；outline/find MUST 按 `ref + display` 计入预算，read MUST 按 `content` 计入预算。
page MUST 是调用位置而不是配置默认值；入口省略 page 时 MUST 固定从 `1` 开始。

#### Scenario: Markdown outline 达到默认限制
- **WHEN** outline 超过默认字符预算
- **THEN** adapter 只返回当前字符预算内的条目
- **THEN** 结果包含下一页 `page: 2`

#### Scenario: 超长 display
- **WHEN** 单条 outline display 超过字符预算
- **THEN** ref 保持完整
- **THEN** display 被压缩以保证分页前进

#### Scenario: ref 超过字符预算
- **WHEN** 单条 outline 或 find 记录的完整 ref 已超过 `limit_chars`
- **THEN** ref 仍保持完整
- **THEN** 该单条记录可超出预算但必须被消耗，后续 page 继续前进

### Requirement: 每个 CLI 拥有独立配置域
每个可执行 CLI MUST 只读取自身配置域，并 MUST 使用“显式参数 > 项目级 CLI 配置 > 用户级 CLI 配置 > 内置默认值”的优先级。
每个 CLI 配置域 MAY 调整本 CLI 的阅读文本模板、guidance、usage 和错误建议；完整协议字段、字段类型和错误 code MUST 保持稳定，readable-json 和 MCP structuredContent MUST 保持 documented shape。

#### Scenario: 启动 invoke
- **WHEN** 调用方准备启动 adapter invoke
- **THEN** 调用方已解析最终有限参数并显式写入请求
- **THEN** invoke 请求包含 page 和 limit_chars

### Requirement: Page 统一表达分页
分页操作 MUST 使用 page 表达读取位置和是否还有更多信息；page 为整数时 MUST 可直接用于继续读取，page 为 null 时 MUST 表示当前操作结束。

#### Scenario: MCP read 达到字符预算
- **WHEN** read 达到字符预算
- **THEN** adapter 返回下一页 page
- **THEN** 调用方保持其他语义参数不变并使用该 page

### Requirement: 稳定契约可直接校验
项目 MUST 为原始协议、manifest、probe 和各 operation readable 输出提供独立 JSON Schema，并 MUST 提供完整可解析示例。原始协议 schema MUST 服务机器稳定接口；readable schema MUST 服务示例、工具声明和实现自测。

#### Scenario: 校验同一业务结果
- **WHEN** 审计者校验 invoke 和 MCP 示例
- **THEN** invoke 响应通过 protocol schema，且 response operation 与 result 类型匹配
- **THEN** MCP structuredContent 通过 readable schema
- **THEN** 两者业务语义一致但包装和兼容目标不同

### Requirement: 文档以中文为主要审计语言
面向审计者的说明、设计理由和要求 MUST 以中文为主，命令、字段、枚举和错误码保留英文机器标识。

#### Scenario: 审计协议
- **WHEN** 审计者阅读协议文档
- **THEN** 规则说明使用中文且机器标识保持英文

### Requirement: 文档契约可映射到自动化验证
测试策略 MUST 将协议稳定性、阅读输出信息密度、配置隔离、ref、page、`docnav` 端到端链路和 MCP bridge 映射到自动化测试。

#### Scenario: 提出实现变更
- **WHEN** 实现者依据 v0 文档规划测试
- **THEN** 能识别 protocol、readable、配置隔离、`docnav` 路由和 MCP 映射测试

### Requirement: 文档阅读路径清晰
README MUST 作为传统项目 README，MUST 包含项目简介、v0 范围、核心流程、Quick Start、验证入口和文档入口；README MUST NOT 作为完整规范副本重复定义细则。
docs/navigation.md MUST 作为日常文档导航入口，MUST 包含术语、角色化阅读路径、文档分层和规则 owner 索引。
Schema 和示例 MUST 作为校验材料；OpenSpec change MUST 作为变更依据、验收和审计历史，不作为日常实现主入口。

#### Scenario: 新实现者选择阅读路径
- **WHEN** 新实现者打开 README
- **THEN** 能了解项目目标、v0 范围、核心流程、Quick Start、验证入口和文档入口
- **THEN** 能进入 docs/navigation.md 按角色找到应阅读的主规范文档
- **THEN** 能通过 docs/navigation.md 区分主规范、校验材料、参考材料和变更历史

#### Scenario: 维护重复规则
- **WHEN** 某条关键规则需要修改
- **THEN** 实现者能在 docs/navigation.md 的规则 owner 索引中定位主规范
- **THEN** 其它文档只引用或摘要该规则

### Requirement: Ref 文档必须描述 adapter 拥有的 ref 边界
Ref 文档和示例 MUST 把 ref 描述为 adapter 生成和解释的非空 opaque string。共享协议、`docnav` 和接入层 MUST 只校验共享字段 shape，并将 ref 原样传给选定 adapter。

ref 的 grammar、定位或查询含义、适用 operation、读取粒度、唯一性、稳定性、消歧、多对一或一对多关系、文档变化后的行为，以及非法或未匹配 ref 的处理 MUST 由对应 adapter 的规范或专属文档定义。共享文档 MUST 通过链接指向对应主文档，不得复制 adapter 私有语义。

共享契约 MUST 强制保留 `outline/find -> ref -> read` 调用流程：adapter 在 outline 或 find 中生成 ref；调用方将相同 path 和 ref 原样提交给 read；core 根据 path 选择 adapter 并原样传递 ref；adapter 返回读取结果或稳定错误。

该流程保证不得解释为共享层保证 read 接受、完整消费、唯一定位、成功读取或返回特定区域。adapter 保留接受、拒绝和解释 ref 的全部权力，并在专属契约中定义结果语义。

该边界 MUST 表达为正确性责任分层，而不是放弃正确性。共享层 MUST 负责按 path 选择 adapter、保持 ref 原值并一致映射稳定错误；adapter MUST 负责其 ref 生成、解释、定位和失败行为符合自身公开契约。共享层 MUST NOT 在不了解 adapter grammar、文档状态和定位模型时替 adapter 建立读取成功或唯一定位保证。

#### Scenario: 共享层原样传递 ref
- **WHEN** 调用方把非空 ref 作为 read 参数提交
- **THEN** `docnav` 根据 path 选择 adapter 并原样传入 ref
- **THEN** `docnav`、MCP、共享协议和 schema 不解析或推断 ref 内部结构
- **THEN** adapter 按其自有契约解释或拒绝该 ref

#### Scenario: 共享调用链保持稳定
- **WHEN** 调用方取得 outline 或 find 返回的 ref
- **THEN** 调用方可以将相同 path 和 ref 原样提交给 read
- **THEN** core 选择 adapter 并原样传递 ref
- **THEN** adapter 返回读取结果或规范允许的稳定错误
- **THEN** 该流程不承诺 read 成功、唯一定位或返回特定区域

#### Scenario: 正确性责任按所有权分层
- **WHEN** 调用方将 outline 或 find 返回的 ref 提交给 read
- **THEN** core 按 path 选择 adapter、保持 ref 原值并一致映射 adapter 返回的稳定错误
- **THEN** adapter 按其专属契约生成、解释、定位或拒绝该 ref
- **THEN** 共享层不替 adapter 声明读取成功、唯一定位或特定区域保证

#### Scenario: 查找 adapter 私有 ref 语义
- **WHEN** 读者需要了解某个 adapter 的 ref 行为
- **THEN** 共享 Ref 文档将读者指向该 adapter 的主文档
- **THEN** 该 adapter 文档说明 grammar、适用 operation、保证范围和错误边界

#### Scenario: Adapter 可以选择不同定位保证
- **WHEN** 两个 adapter 为各自格式设计 ref
- **THEN** 一个 adapter 可以选择唯一定位，另一个 adapter 可以选择非唯一、部分消费或其它语义
- **THEN** 两者只需满足共享的非空字符串载体和原样传递边界

### Requirement: Rust CLI 参数解析必须服务 AI 维护和一次成功调用
Rust CLI 参数解析文档 MUST 将 `clap` 定义为本 change 覆盖的核心 `docnav` document CLI、已存在 core non-document 命令、adapter direct CLI、help 和后续 Rust document CLI 入口的首选 argv 结构解析基础。

文档必须描述以下稳定规则：

- 核心 `docnav` document CLI argv、adapter direct CLI document argv 和 adapter `invoke` JSON 在各自传输层解析成功后映射为 canonical document operation input 或等价 semantic request。
- canonical document operation input 或等价 semantic request 是 Rust 边界后的内部语义输入模型，不是 protocol envelope、schema 稳定类型或 MCP 传输模型；实现可以使用等价内部类型，不要求共享同一个 Rust 类型。
- 传输层解析成功后，文档操作行为由共享语义归一、容错校验和 operation 执行负责。
- 当前 operation 的必需语义输入存在且实际使用参数有效时，即使 argv 包含未知 flag、多余 positional 或当前 operation 不使用的参数，CLI argv 也应继续成功执行。
- 当前 operation 实际使用的参数保持严格。
- Adapter `invoke` 在进入 canonical document operation input 或等价 semantic request 前拒绝 malformed JSON、未知字段、缺失字段和类型错误。
- `clap` 是 CLI argv 结构解析基础；Docnav 在确定 command/operation 后只对当前 operation 实际使用的参数做类型、范围和枚举校验，并受控收集 unknown、extra positional 和 unused known 参数的原始 token 生成 warning metadata，但不在收集层复制业务参数解释、默认值归一或 request 构造逻辑。
- `clap` 和受控宽松收集不改变 protocol 字段、ref ownership、adapter 格式解析 ownership 或 MCP 传输语义。
- `docnav-mcp` 将 MCP tool call 映射到核心 `docnav` CLI，并包装 MCP 输出；它不进入 adapter SDK 解析或 adapter `invoke`。
- Readable warning 使用稳定 warning envelope：稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象。
- 当前 warning family marker 包括 `cli_argv_ignored` 和 `adapter_candidate_failure`。
- CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不稳定。
- 文档和测试策略维护命令族矩阵，固定每类命令是否进入 canonical document operation input 或等价 semantic request、是否启用宽松 argv、ignored argv 诊断通道、protocol-shaped stdout 边界和 help 是否执行业务。
- 核心 `docnav adapter list/install/update/remove` 管理命令不属于本 change 的 CLI parser 迁移或验收范围；命令族矩阵只记录其 owner、边界和非验收状态。

#### Scenario: 入口解析分层但共享文档操作语义管道
- **WHEN** 核心 `docnav` document CLI argv、adapter direct CLI document argv 或 adapter `invoke` JSON 分别通过各自入口解析
- **AND** 传输层校验成功
- **THEN** 文档操作请求映射为 canonical document operation input 或等价 semantic request
- **THEN** 默认值、必需语义检查、native options、request 构造和 operation handler 使用共享逻辑
- **THEN** 不因入口不同复制业务参数规则

#### Scenario: AI 调用包含未知参数但必需语义完整
- **WHEN** AI agent 调用 Rust CLI 时传入未知 flag、多余 positional 或当前 operation 不使用的参数
- **AND** path、ref、query、page、limit_chars、output 等当前 operation 实际需要的语义可被解析并通过校验
- **THEN** CLI 继续执行对应 operation
- **THEN** 被忽略输入产生阅读层 warning 或 stderr 诊断
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`
- **THEN** protocol envelope、MCP transport ownership 和 adapter ref 语义保持不变

#### Scenario: unused known 参数不触发 eager typed failure
- **WHEN** Rust CLI 收到当前 operation 不使用的 known 参数
- **AND** 该参数值无法通过其它 operation 的类型、范围或枚举校验
- **AND** 当前 operation 实际需要的语义仍可被解析并通过校验
- **THEN** CLI 继续执行当前 operation
- **THEN** 该参数以原始 token 进入 warning metadata 或 stderr 诊断
- **THEN** CLI 不因其它 operation 的参数规则提前失败

#### Scenario: 命令族矩阵固定 CLI 行为边界
- **WHEN** 实现者更新 Rust CLI argv parsing、warning behavior、help 行为或 protocol-shaped stdout 边界
- **THEN** `docs/cli.md` 和 `docs/testing.md` 包含命令族矩阵
- **THEN** 矩阵至少覆盖 core document operations、core non-document commands、core adapter management commands、adapter direct document operations、adapter direct machine commands、help commands 和 MCP bridge
- **THEN** 每个命令族标明 owner、是否进入 canonical document operation input 或等价 semantic request、是否启用宽松 argv、ignored argv 诊断通道、protocol-shaped stdout 边界、help 是否执行业务，以及是否属于本 change 验收范围
- **THEN** 核心 `docnav adapter list/install/update/remove` 管理命令只记录 owner、边界和非验收状态，不作为本 change 的实现或验收项

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** Rust CLI 收到当前 operation 实际使用的已知参数
- **AND** 该参数缺少必需值或值无法通过类型、枚举或范围校验
- **THEN** CLI 返回输入错误
- **THEN** CLI 不通过宽松 argv 策略静默替换为默认值

#### Scenario: 阅读层 warning envelope 保留
- **WHEN** CLI 阅读层输出承载 warning
- **THEN** readable-json 或 MCP structuredContent 中的 warning item 包含稳定 `id`、非空 `reason`、稳定 `effect` 和 `details` 对象
- **THEN** CLI argv warning 使用 `id: "cli_argv_ignored"`，相关 argv token 只作为 `details.tokens` 等 family-specific detail 表达
- **THEN** adapter candidate warning 使用 `id: "adapter_candidate_failure"`，并在 `details` 中保留 `adapter_id`、`stage`、`code` 和可选 `preselected` 等 candidate family 字段
- **THEN** 测试只要求 stable envelope、family-specific 字段和用户可理解诊断
- **THEN** 测试不要求 exact token 分组、`reason` 文案或 token 消费顺序

#### Scenario: Invoke 入口仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段、缺少必需字段或参数类型错误的 JSON request
- **THEN** invoke 按 protocol schema 返回结构化失败
- **THEN** 该请求不进入 canonical document operation input 或等价 semantic request
- **THEN** CLI argv 容错规则不用于忽略该 JSON request 的错误字段

#### Scenario: 有效 invoke 请求共享 operation 处理
- **WHEN** adapter `invoke` 从 stdin 收到 schema-valid 的 outline/read/find/info request
- **THEN** request 被映射为 canonical document operation input 或等价 semantic request
- **THEN** 共享语义归一和统一 operation handler 负责后续处理
- **THEN** 不使用独立于 direct CLI 的业务参数解释规则

#### Scenario: MCP ownership 保持接入层边界
- **WHEN** `docnav-mcp` 收到 document tool call
- **THEN** `docnav-mcp` 将 tool arguments 映射为核心 `docnav` CLI 调用
- **THEN** `docnav-mcp` 不直接调用 adapter SDK、adapter `invoke` 或 Rust CLI argv parser
- **THEN** MCP structuredContent 只复用 readable schema/outputSchema 中的 stable warning envelope

#### Scenario: 文档 owner 同步 CLI 解析规则
- **WHEN** 实现者更新 CLI argv parsing、warning behavior 或 `clap` dependency
- **THEN** `docs/cli.md` 描述用户可见 CLI 行为、stable warning envelope、当前 operation 使用参数严格性、宽松 argv 边界和 help 验收
- **THEN** `docs/adapter-contract.md` 描述 adapter direct CLI、invoke strict transport validation 和 canonical document operation input 或等价 semantic request 共享边界
- **THEN** `docs/testing.md` 描述命令族矩阵、成功路径、必要失败、help 可用、共享语义归一、stable warning envelope 和 schema 边界验证
- **THEN** `docs/schemas/readable-common.schema.json`、MCP outputSchema 示例和 `docs/examples/json/mcp-*-tool.json` 描述 stable warning envelope、当前 warning family marker、稳定 effect 和 family-specific details
- **THEN** `docs/protocol.md` 如涉及 warning 或 protocol-shaped stdout 边界，只说明 protocol response、manifest 和 probe 不承载 CLI warning fields

### Requirement: 已实现 adapter 的私有行为必须有独立主文档
文档导航 MUST 为需要长期维护私有导航行为的已实现 adapter 指向独立主文档。

Markdown v0 adapter MUST 使用 `docs/adapters/markdown.md` 记录当前实现的导航行为、ref grammar、保证范围、错误分类和验证入口。`docs/reference-materials/markdown-navigator.md` MUST 只记录外部来源和迁移依据。

#### Scenario: 阅读 Markdown adapter 契约
- **WHEN** 实现者或审计者需要了解 Markdown adapter 的私有导航行为
- **THEN** 文档导航将其指向 `docs/adapters/markdown.md`
- **THEN** 共享 Ref、架构、协议和 adapter contract 文档只保留共享边界和链接

#### Scenario: 其它 adapter 建立自己的主文档
- **WHEN** 其它格式 adapter 需要长期维护格式私有行为
- **THEN** 该 adapter 使用自己的规范或专属文档记录这些行为
- **THEN** 其设计不继承 Markdown adapter 的 grammar、唯一性、稳定性或消歧语义

### Requirement: 稳定错误必须支持 adapter 报告非法 ref
原始协议和阅读错误层 MUST 支持稳定错误 `REF_INVALID`。该错误 MUST 表示请求中的 ref 是非空字符串且请求传输 shape 有效，但选定 adapter 无法按其当前私有 grammar 解释该 ref。

`REF_INVALID` 的稳定 details MUST 包含 `ref` 和 `reason`。具体 adapter MUST 自行决定哪些输入属于非法 grammar，并在其专属规范中说明 `REF_INVALID` 与其它 ref 错误的边界。

#### Scenario: Adapter 拒绝非法 ref grammar
- **WHEN** read 请求通过共享 schema 校验
- **AND** 选定 adapter 判定 ref 不符合其当前 grammar
- **THEN** adapter 返回 `REF_INVALID`
- **THEN** error details 包含原始 `ref` 和非空 `reason`

#### Scenario: 共享层不解析 adapter grammar
- **WHEN** `docnav` 或 MCP 接入层收到非空 ref
- **THEN** 共享层将 ref 原样传给选定 adapter
- **THEN** `docnav` 和 MCP 不自行判断该 ref 是否符合 adapter grammar

### Requirement: 共享 Rust crate 所有权必须保持 Docnav 契约分层

Docnav 共享 Rust crate MUST 保持原始协议、document output 编排、JSON IO、readable renderer、diagnostics、direct CLI argv compatibility、adapter SDK 行为和格式 adapter 语义之间的既有分层。共享 crate MUST 只上移稳定契约和机械流程；routing、ref interpretation、format parsing、process runtime 和用户可见 surface policy 等 owner-specific 判断 MUST 留在既有 owner，除非后续 spec 明确改变。

#### Scenario: 共享 crate 依赖方向保持无环

- **WHEN** 实现新增 `docnav-diagnostics`、`docnav-cli-args`、`docnav-json-io` 或 `docnav-output`
- **THEN** 这些 crate 只依赖其契约所需的下层共享 crate
- **THEN** `docnav-json-io` 不依赖 `docnav-output`、`docnav-readable`、`docnav` core 或 `docnav-adapter-sdk`
- **THEN** `docnav-output` 可以依赖 `docnav-json-io`，但不依赖 `docnav` core 或 `docnav-adapter-sdk`
- **THEN** `docnav` core 和 `docnav-adapter-sdk` 可以依赖 `docnav-output`
- **THEN** 格式 adapter 不需要依赖 `docnav` core 即可产生 direct CLI document output

#### Scenario: 共享 crate 不接管 adapter-owned 语义

- **WHEN** adapter 生成、解析或拒绝一个 ref
- **THEN** 共享 protocol、output、diagnostics 和 CLI argv crate 将该 ref 视为 opaque value
- **THEN** 共享 crate 不推断 heading structure、唯一性、region boundary 或格式专属 navigation behavior

#### Scenario: 文档先行约束先于代码迁移

- **WHEN** 本 change 进入实现
- **THEN** 实现者先同步主规范、schema/example/fixture/testing 文档中的 owner 和验证说明
- **THEN** crate 新增或代码迁移在主规范和验证材料对齐之后开始
- **THEN** 代码实现不得作为定义共享 crate contract 的第一来源

#### Scenario: 本 change 的共享 crate 集合保持限定

- **WHEN** 本 change 进入实现
- **THEN** 不为 core path display normalization 引入 path utility crate
- **THEN** 不为 adapter process startup 或 registry command path handling 引入 process runner crate
- **THEN** 不为 manifest/probe/invoke ownership 引入 adapter boundary crate
- **THEN** `docnav-json-io` 不成为 schema、manifest/probe 或 document output policy owner

