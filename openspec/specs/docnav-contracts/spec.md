# docnav-contracts Specification

## Purpose
定义 Docnav v0 长期文档契约，约束原始协议、阅读输出、职责边界、阅读路径、示例材料和自动化验证映射。
## Requirements
### Requirement: 原始协议与阅读输出分层
项目 MUST 将 adapter invoke 原始协议与 CLI 阅读输出定义为两个独立语义层。原始协议层 MUST 作为机器稳定接口；阅读输出层 MUST 优先服务 AI 和人类阅读，不作为长期机器解析接口。两层 MUST 复用业务语义，并以各自 schema 校验字段形状。

#### Scenario: 原始协议响应自描述 operation
- **WHEN** adapter invoke 返回成功响应
- **THEN** 响应 envelope 包含与请求一致的 `operation`
- **THEN** protocol response schema 使用 `operation` 校验对应 result 类型

#### Scenario: Read 保留内容类型
- **WHEN** adapter read 返回内容
- **THEN** protocol result 和 readable JSON 均包含 `content_type`
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
- **WHEN** 调用方提供 `--format markdown`
- **THEN** `docnav` 优先使用 manifest 中匹配 `formats[].id` 的 adapter 候选并执行校验
- **THEN** 校验失败时继续扩展名匹配阶段

#### Scenario: 显式 content type 优先
- **WHEN** 调用方提供 `--format text/markdown`
- **THEN** `docnav` 优先使用 manifest 中匹配 `formats[].content_types[]` 的 adapter 候选并执行校验
- **THEN** 校验成功时选中该 adapter

#### Scenario: 扩展名和全量 probe
- **WHEN** 显式格式阶段未选中 adapter
- **THEN** `docnav` 按 manifest `formats[].extensions[]` 匹配 path 扩展名并校验候选 adapter
- **THEN** 扩展名候选都失败时，`docnav` 逐个 probe 已安装 adapter
- **THEN** 全部阶段失败时返回 `FORMAT_UNKNOWN`

### Requirement: 接入方式共享 `docnav` 契约
直接 CLI、skill 和 AGENTS.md / system prompt MUST 共享 `docnav` 的 path、ref、page、limit_chars、输出模式和错误契约。

#### Scenario: Agent 通过项目规则读取文档
- **WHEN** 项目规则提示 agent 读取大型文档
- **THEN** agent 使用 `docnav outline/read/find/info`
- **THEN** agent 将 outline 返回的 ref 原样传入 read

### Requirement: Markdown v0 实现全部首期能力
Markdown v0 adapter MUST 实现 `outline`、`read`、`find` 和 `info` 全部能力；`outline -> ref -> read` MUST 作为首要纵向阅读链路。

#### Scenario: Markdown capability 集合
- **WHEN** 调用方读取 Markdown adapter manifest
- **THEN** `capabilities` 包含 `outline`、`read`、`find` 和 `info`
- **THEN** 每个能力都有对应 CLI 和 invoke 映射验证

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
每个 CLI 配置域 MAY 调整本 CLI 的阅读文本模板、guidance、usage 和错误建议；完整协议字段、字段类型和错误 code MUST 保持稳定，readable-json MUST 保持 documented shape。

#### Scenario: 启动 invoke
- **WHEN** 调用方准备启动 adapter invoke
- **THEN** 调用方已解析最终有限参数并显式写入请求
- **THEN** invoke 请求包含 page 和 limit_chars

### Requirement: Page 统一表达分页
分页操作 MUST 使用 page 表达读取位置和是否还有更多信息；page 为整数时 MUST 可直接用于继续读取，page 为 null 时 MUST 表示当前操作结束。

#### Scenario: read 达到字符预算
- **WHEN** read 达到字符预算
- **THEN** adapter 返回下一页 page
- **THEN** 调用方保持其他语义参数不变并使用该 page

### Requirement: 稳定契约可直接校验
项目 MUST 为原始协议、manifest、probe 和各 operation readable 输出提供独立 JSON Schema，并 MUST 提供完整可解析示例。原始协议 schema MUST 服务机器稳定接口；readable schema MUST 服务示例和实现自测。

#### Scenario: 校验同一业务结果
- **WHEN** 审计者校验 protocol 和 readable 示例
- **THEN** invoke 响应通过 protocol schema，且 response operation 与 result 类型匹配
- **THEN** readable JSON 通过 readable schema
- **THEN** 两者业务语义一致但包装和兼容目标不同

### Requirement: 文档以中文为主要审计语言
面向审计者的说明、设计理由和要求 MUST 以中文为主，命令、字段、枚举和错误码保留英文机器标识。

#### Scenario: 审计协议
- **WHEN** 审计者阅读协议文档
- **THEN** 规则说明使用中文且机器标识保持英文

### Requirement: 文档契约可映射到自动化验证
测试策略 MUST 将协议稳定性、阅读输出信息密度、配置隔离、ref、page 和 `docnav` 端到端链路映射到自动化测试。

#### Scenario: 提出实现变更
- **WHEN** 实现者依据 v0 文档规划测试
- **THEN** 能识别 protocol、readable、配置隔离和 `docnav` 路由测试

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
- **THEN** `docnav`、共享协议和 schema 不解析或推断 ref 内部结构
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

### Requirement: Rust CLI 参数解析必须服务 AI 可修复调用
Rust CLI 参数解析文档 MUST 将 `clap` 定义为本 change 覆盖的核心 `docnav` document CLI、已存在 core non-document 命令、adapter direct CLI、help 和后续 Rust document CLI 入口的首选 argv 结构解析基础。

文档必须描述以下稳定规则：

- 核心 `docnav` document CLI argv、adapter direct CLI document argv 和已解码的 adapter `invoke` JSON 都作为 direct input 进入标准参数/typed-field processing，再映射为 canonical document operation input 或等价 semantic request。
- canonical document operation input 或等价 semantic request 是 Rust 边界后的内部语义输入模型，不是 protocol envelope 或 schema 稳定类型；实现可以使用等价内部类型，不要求共享同一个 Rust 类型。
- 传输层解析成功后，文档操作行为由共享语义归一、strict 校验和 operation 执行负责。
- 未知 flag、多余 positional 或当前 operation 不使用的参数 MUST 在 owning boundary 形成 primary input diagnostic。
- 当前 operation 实际使用的参数保持严格。
- Adapter `invoke` 只把 malformed JSON 作为 transport decode failure；未知字段、缺失字段和类型错误由标准参数/typed-field processing 产生诊断，且不得产出 canonical document operation input 或等价 semantic request。
- `clap` 是 CLI argv 结构解析基础；Docnav 在确定 command/operation 后只允许当前 operation owning boundary 使用的参数进入 semantic request，但不在收集层复制业务参数解释、默认值归一或 request 构造逻辑。
- `clap` 和 strict 收集不改变 protocol 字段、ref ownership 或 adapter 格式解析 ownership。
- 文档和测试策略维护命令族矩阵，固定每类命令是否进入 canonical document operation input 或等价 semantic request、strict argv 边界、invalid argv 诊断通道、protocol-shaped stdout 边界和 help 是否执行业务。
- 核心 `docnav adapter list/install/update/remove` 管理命令不属于本 change 的 CLI parser 迁移或验收范围；命令族矩阵只记录其 owner、边界和非验收状态。

#### Scenario: 入口解析分层但共享文档操作语义管道
- **WHEN** 核心 `docnav` document CLI argv、adapter direct CLI document argv 或 adapter `invoke` JSON 分别通过各自入口解析
- **AND** 传输层校验成功
- **THEN** 文档操作请求映射为 canonical document operation input 或等价 semantic request
- **THEN** 默认值、必需语义检查、native options、request 构造和 operation handler 使用共享逻辑
- **THEN** 不因入口不同复制业务参数规则

#### Scenario: AI 调用包含未知参数时失败且可修复
- **WHEN** AI agent 调用 Rust CLI 时传入未知 flag、多余 positional 或当前 operation 不使用的参数
- **AND** path、ref、query、page、limit_chars、output 等当前 operation 实际需要的语义可被解析并通过校验
- **THEN** CLI 返回 primary input diagnostic
- **THEN** CLI 不执行对应 operation
- **THEN** protocol envelope 和 adapter ref 语义保持不变

#### Scenario: unused known 参数触发 input diagnostic
- **WHEN** Rust CLI 收到当前 operation 不使用的 known 参数
- **AND** 该参数值无法通过其它 operation 的类型、范围或枚举校验
- **AND** 当前 operation 实际需要的语义仍可被解析并通过校验
- **THEN** CLI 返回 primary input diagnostic
- **THEN** CLI 不执行当前 operation

#### Scenario: 命令族矩阵固定 CLI 行为边界
- **WHEN** 实现者更新 Rust CLI argv parsing、diagnostic behavior、help 行为或 protocol-shaped stdout 边界
- **THEN** `docs/cli.md` 和 `docs/testing.md` 包含命令族矩阵
- **THEN** 矩阵至少覆盖 core document operations、core non-document commands、core adapter management commands、adapter direct document operations、adapter direct machine commands 和 help commands
- **THEN** 每个命令族标明 owner、是否进入 canonical document operation input 或等价 semantic request、strict argv 边界、invalid argv 诊断通道、protocol-shaped stdout 边界、help 是否执行业务，以及是否属于本 change 验收范围
- **THEN** 核心 `docnav adapter list/install/update/remove` 管理命令只记录 owner、边界和非验收状态，不作为本 change 的实现或验收项

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** Rust CLI 收到当前 operation 实际使用的已知参数
- **AND** 该参数缺少必需值或值无法通过类型、枚举或范围校验
- **THEN** CLI 返回输入错误
- **THEN** CLI 不通过 strict argv 策略静默替换为默认值

#### Scenario: Invoke 入口仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段、缺少必需字段或参数类型错误的 JSON request
- **THEN** invoke 通过标准参数/typed-field processing 返回结构化失败
- **THEN** 该请求不产出 canonical document operation input 或等价 semantic request
- **THEN** CLI argv 容错规则不用于忽略该 JSON request 的错误字段

#### Scenario: 有效 invoke 请求共享 operation 处理
- **WHEN** adapter `invoke` 从 stdin 收到可映射为 outline/read/find/info 的 request
- **THEN** request 被映射为 canonical document operation input 或等价 semantic request
- **THEN** 共享语义归一和统一 operation handler 负责后续处理
- **THEN** 不使用独立于 direct CLI 的业务参数解释规则

#### Scenario: 文档 owner 同步 CLI 解析规则
- **WHEN** 实现者更新 CLI argv parsing、diagnostic behavior 或 `clap` dependency
- **THEN** `docs/cli.md` 描述用户可见 CLI 行为、当前 operation 使用参数严格性、strict argv 边界和 help 验收
- **THEN** `docs/adapter-contract.md` 描述 adapter direct CLI、invoke direct input 处理和 canonical document operation input 或等价 semantic request 共享边界
- **THEN** `docs/testing.md` 描述命令族矩阵、成功路径、必要失败、help 可用、共享语义归一和 schema 边界验证
- **THEN** readable error schema 和 examples 描述 primary `DiagnosticRecord` readable projection
- **THEN** `docs/protocol.md` 描述 protocol response、manifest 和 probe 的 schema payload 边界

### Requirement: 已实现 adapter 的私有行为必须有独立主文档
文档导航 MUST 为需要长期维护私有导航行为的已实现 adapter 指向独立主文档。

Markdown v0 adapter MUST 使用 `docs/adapters/markdown.md` 记录当前实现的导航行为、ref grammar、保证范围、错误分类和验证入口。

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
- **WHEN** `docnav` 收到非空 ref
- **THEN** 共享层将 ref 原样传给选定 adapter
- **THEN** `docnav` 不自行判断该 ref 是否符合 adapter grammar

### Requirement: 共享 Rust crate 所有权必须保持 Docnav 契约分层

Docnav 共享 Rust crate MUST 保持原始协议、document output 编排、JSON IO、readable renderer、diagnostics、direct CLI argv classification、adapter SDK 行为和格式 adapter 语义之间的既有分层。共享 crate MUST 只上移稳定契约和机械流程；routing、ref interpretation、format parsing、process runtime 和用户可见 surface policy 等 owner-specific 判断 MUST 留在既有 owner，除非后续 spec 明确改变。

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

### Requirement: Adapter direct CLI 配置域不得被 core 重新解释
每个 adapter direct CLI MUST 只读取自身 adapter id 对应的配置域，并 MAY 暴露由 SDK 拥有的 `--project-config-path <path>` 和 `--user-config-path <path>` 以覆盖项目级和用户级 adapter 配置文件路径。Core `docnav` MUST NOT 读取 adapter direct CLI 配置，MUST NOT 从 adapter direct CLI 配置合成格式专属 `arguments.options`，并 MUST 继续只通过 protocol/request/readable output 边界与 adapter 交互。CLI argv 和 adapter `invoke` stdin JSON 都是入口传参方式；对应 CLI 内部 document operation 线路 MUST 保持业务逻辑唯一来源。

#### Scenario: Core 不读取 Markdown adapter 配置
- **WHEN** `.docnav/docnav-markdown.json` 设置 `options.max_heading_level`
- **AND** 调用方执行 core `docnav outline docs/guide.md`
- **THEN** core `docnav` 不读取该 adapter 配置文件
- **THEN** core `docnav` 不从该配置合成 Markdown `arguments.options`
- **THEN** adapter-specific options 只有在请求中显式存在时才传给 adapter

#### Scenario: 配置路径覆盖只属于 adapter direct CLI
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path fixtures/project.json`
- **THEN** 该路径覆盖只影响本次 adapter direct CLI 配置加载
- **THEN** 路径覆盖不成为 protocol request 字段
- **THEN** core `docnav` 不解释该路径覆盖

### Requirement: Direct CLI 配置读取只提供标准参数来源对象
Adapter direct CLI document operation MUST 读取自身配置源，并把可用配置值合并为标准 direct CLI 参数来源对象。未覆盖的默认配置文件缺失表示对应层没有配置源。显式覆盖配置路径不存在、不可读或不是可读取文件时，adapter direct CLI MUST 返回 blocking config diagnostic。JSON 语法无效或顶层不是 JSON object 的配置源 MUST 返回 blocking config diagnostic。已成功读取的配置内容 MUST 先合并为标准 direct CLI 参数来源对象，再交给既有 direct CLI 参数处理链路完成标准化和校验。配置读取层只负责 JSON 读取、固定字段投影和来源优先级；未知顶层字段或未知 `defaults` 字段不产生配置读取 diagnostic，`options` object 内的 key/value 作为 native options 参数来源交给后续链路处理。Config schema/example MAY 作为填写提示、文档校验或 adapter package 打包材料，但 MUST NOT 成为 adapter direct CLI runtime 读取配置的前置条件。

#### Scenario: 未覆盖默认配置文件缺失时使用下一级默认值
- **WHEN** 项目级或用户级 adapter 配置文件不存在
- **AND** 调用方没有显式覆盖该层配置路径
- **THEN** adapter direct CLI 继续按其余来源解析默认值
- **THEN** 缺失文件不产生配置源输入

#### Scenario: 显式覆盖配置路径缺失时失败
- **WHEN** 调用方显式传入 `--project-config-path missing.json`
- **AND** 该路径不存在或不可读
- **THEN** adapter direct CLI 返回 blocking config diagnostic
- **THEN** operation request 不构造

#### Scenario: Config schema 不作为 runtime gate
- **WHEN** `docs/schemas/docnav-markdown-config.schema.json` 不存在于运行环境
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md`
- **THEN** adapter direct CLI 仍按配置源读取、字段投影和标准参数处理链路处理本次调用
- **THEN** runtime 不要求先加载 config schema

#### Scenario: 未知配置字段不属于配置读取错误
- **WHEN** adapter direct CLI 读取到 schema 未声明但 JSON 语法有效且顶层为 object 的配置字段
- **THEN** adapter direct CLI 不把该字段视为配置源读取失败
- **THEN** 已知字段仍按固定投影和来源优先级参与标准 direct CLI 参数来源对象合并
- **THEN** `options` object 内的 key/value 是否可用由后续 direct CLI 参数处理链路决定

#### Scenario: JSON 语法无效时失败
- **WHEN** adapter direct CLI 读取到语法无效的 JSON 配置文件
- **AND** 用户级 adapter 配置包含 `defaults.output: "readable-json"`
- **AND** 调用方未显式传入 `--output`
- **THEN** adapter direct CLI 返回 blocking config diagnostic
- **THEN** operation request 不构造

### Requirement: Runtime problems flow through a request-local diagnostic stack

Docnav runtime and public surface code MUST record runtime problems in a request-local diagnostic stack before the owning boundary decides whether to continue, fail, exit, or write surface-specific output.

#### Scenario: Internal discovery problem is recorded before continuation

- **WHEN** automatic adapter discovery encounters skipped adapter candidate evidence
- **THEN** the condition is pushed as a diagnostic record before the caller decides how to proceed
- **THEN** execution continues only because the candidate was not caller-declared explicit input
- **THEN** the record remains available until the boundary owner reads, renders, or flushes it

#### Scenario: Fatal problem records context before failure surface

- **WHEN** an operation encounters a fatal request, document, adapter boundary, or internal failure
- **THEN** the diagnostic stack records the fatal context before the fatal outcome is returned or propagated
- **THEN** the record carries a diagnostic code that can be projected to the target surface error code, message, details, guidance, and exit-code category

#### Scenario: Diagnostic stack stores facts only

- **WHEN** a diagnostic record is pushed
- **THEN** the stack stores the record without deciding whether the operation succeeds or fails
- **THEN** the caller or surface owner decides continuation, failure, output format, output channel, and exit behavior

### Requirement: DiagnosticCode owns identity and canonical details

`docnav-diagnostics` MUST own `DiagnosticCode`, grouped code families, each code's canonical details object, and projection metadata. Other Docnav crates MUST use those diagnostics-owned identities and MUST NOT redefine protocol, readable, adapter, or standard-parameter diagnostic code identities.

#### Scenario: Diagnostic code aggregates grouped enums

- **WHEN** implementation groups diagnostic codes by purpose, producer, or projection family
- **THEN** each group can use its own manually maintained enum
- **THEN** the top-level `DiagnosticCode` aggregates those group enums into one mechanical identity domain
- **THEN** callers outside `docnav-diagnostics` use the top-level `DiagnosticCode` or its explicit family conversions

#### Scenario: Diagnostic code owns surface error identity

- **WHEN** a diagnostic record is rendered as a fatal error, protocol error code, stderr line, readable error field, or other surface field
- **THEN** the mechanical identity is derived from the record's `DiagnosticCode`
- **THEN** the surface field does not become the source of identity for the internal channel

#### Scenario: Diagnostic code owns canonical details

- **WHEN** a caller creates a diagnostic record for a specific `DiagnosticCode`
- **THEN** the record details conform to the canonical details object structure for that code
- **THEN** missing required fields, wrong field types, or disallowed extra fields are rejected by the diagnostics-owned constructor or checker
- **THEN** surface projection maps from that canonical details object

#### Scenario: Diagnostic code carries projection rules

- **WHEN** implementation defines whether a diagnostic can project to an error surface, stderr line, or exit behavior
- **THEN** the rule is derived from the `DiagnosticCode` rule set
- **THEN** protocol schema, readable schema, examples, and fixtures consume the projection but do not own the rule source

### Requirement: Diagnostic stack provides scoped checkpoints and LIFO retrieval

The diagnostic stack MUST provide request-scoped ids, checkpoints, and default LIFO retrieval so callers can inspect or drain diagnostics created after a known point without exposing stack implementation details as public output contract.

#### Scenario: Pushed record can be retrieved by id

- **WHEN** a caller pushes a diagnostic record onto the stack
- **THEN** the stack returns an opaque `DiagnosticId` scoped to that stack lifetime
- **THEN** a caller holding that id can retrieve the same record without popping or consuming it
- **THEN** callers cannot provide their own `DiagnosticId` value when pushing

#### Scenario: Caller drains records after a checkpoint

- **WHEN** a caller creates a mark before trying a candidate path
- **AND** that candidate path pushes one or more diagnostic records
- **THEN** the caller can drain records pushed after the mark as a batch
- **THEN** records that existed before the mark remain in the stack

#### Scenario: Caller drains records after a record id

- **WHEN** a caller holds the `DiagnosticId` for an earlier stack record
- **THEN** the caller can choose whether draining starts strictly after that record id or includes the record referenced by that id
- **THEN** the record referenced by the id remains available unless the caller explicitly removes it

#### Scenario: Stack retrieval is LIFO by default

- **WHEN** a caller pops, drains, snapshots, renders, or flushes stack records without requesting another order
- **THEN** the stack returns records in last-in-first-out order
- **THEN** a caller that needs insertion order or grouped output explicitly reverses or regroups the returned records outside the stack

#### Scenario: Stack lifetime does not cross process boundary

- **WHEN** a top-level `docnav` command, adapter direct command, or adapter `invoke` request creates a diagnostic stack
- **THEN** stack ids and marks are valid only for that in-process stack lifetime
- **THEN** serialized protocol/readable output does not expose stack ids, marks, or indexes as durable refs

### Requirement: Boundary surfaces project diagnostic records

Modules that discover problems MUST push diagnostic records into the channel. Boundary surfaces MUST read those records and project them according to their owner contract.

#### Scenario: Runtime module writes but does not format final output

- **WHEN** core runtime, adapter routing, standard parameter resolution, adapter direct CLI, or adapter `invoke` handling discovers a problem
- **THEN** that module records what happened, its impact, canonical details, and source in the diagnostic stack
- **THEN** that module does not own final user-visible formatting unless it is also the boundary surface owner

#### Scenario: Boundary surface projects records to its own contract

- **WHEN** CLI, protocol surface, readable output, adapter direct CLI, or adapter `invoke` handler reaches an output boundary
- **THEN** the boundary reads the diagnostic stack records relevant to that output
- **THEN** the boundary projects records according to `docs/cli.md`, `docs/protocol.md`, `docs/output.md`, or `docs/adapter-contract.md`

#### Scenario: Surface docs do not redefine internal channel semantics

- **WHEN** protocol, readable, CLI, adapter, schema, or example docs describe diagnostic output
- **THEN** they define display, filtering, mapping, stdout/stderr placement, envelope shape, or exit behavior for their surface
- **THEN** they do not redefine `DiagnosticCode`, canonical details, `DiagnosticId`, mark lifetime, or default LIFO semantics

### Requirement: Legacy diagnostic sources are fully migrated

Existing error fact sources MUST fully migrate to diagnostic channel records and diagnostics-owned projections. The completed implementation MUST NOT retain a legacy parallel diagnostic fact source.

#### Scenario: Stable error projection uses diagnostic code

- **WHEN** a fatal diagnostic is rendered or serialized as a stable surface error
- **THEN** the target surface error code is derived from `DiagnosticCode`
- **THEN** no legacy stable error object remains as an owning fact model after migration completes

#### Scenario: Direct stderr path records before flushing

- **WHEN** a Rust entry point rejects command shape, fails manifest/probe/invoke decode, fails schema validation, or hits output write failure before normal document output
- **THEN** the entry point records diagnostic context in the channel before writing stderr or protocol/readable failure output
- **THEN** the observable output follows the owning surface projection policy

### Requirement: Protocol error rules JSON is removed

`docs/protocol/error-rules.json` MUST be deleted as a rule source. Protocol error code and details validation MUST consume `DiagnosticCode` protocol projections from `docnav-diagnostics`, while protocol docs, schema, examples, and tests remain validation and presentation materials.

#### Scenario: Protocol crate uses diagnostics code directly

- **WHEN** `docnav-protocol` needs to render, validate, or categorize a protocol-visible diagnostic
- **THEN** it depends on `docnav-diagnostics` and uses `DiagnosticCode` or an explicit diagnostics-owned protocol projection
- **THEN** it does not maintain a separate `StableErrorCode` or protocol-local required-details rule source

#### Scenario: Protocol schema consumes projection

- **WHEN** `protocol-response.schema.json` validates an error response
- **THEN** its code enum and details constraints match the diagnostics-owned protocol projection
- **THEN** the schema does not define new diagnostic code or details rules that are absent from `DiagnosticCode`

#### Scenario: Error rules generator no longer reads protocol JSON

- **WHEN** repository validation checks generated error rules
- **THEN** no script reads `docs/protocol/error-rules.json`
- **THEN** generated Rust or TypeScript constants, if any remain, are derived from diagnostics-owned rules or are replaced by direct `docnav-diagnostics` usage

### Requirement: Diagnostic channel changes update validation materials

Changes to diagnostic channel semantics or surface projection MUST update the relevant owner docs, JSON Schema, examples, fixtures, and tests in the same implementation work.

#### Scenario: Protocol JSON projection is validated

- **WHEN** a document operation is rendered as `protocol-json`
- **THEN** stdout follows the protocol response schema owned by the protocol docs
- **THEN** any protocol-visible diagnostic fields or errors are derived from diagnostic channel records

#### Scenario: Readable output projection is validated

- **WHEN** a document operation is rendered as `readable-view` or `readable-json`
- **THEN** primary failure diagnostics are rendered from diagnostic channel records when the operation fails
- **THEN** readable output remains separate from the protocol response envelope

#### Scenario: Adapter machine command projection is validated

- **WHEN** an adapter direct `manifest`, `probe`, or `invoke` command writes machine output
- **THEN** stdout follows the owning manifest, probe, or protocol response schema
- **THEN** diagnostic output is derived from diagnostic channel records according to that surface policy

#### Scenario: Diagnostic effect requires its own behavior owner

- **WHEN** implementation updates diagnostic effects while applying this change
- **THEN** it adds only effects required by a concrete behavior owner and matching validation material

### Requirement: Raw protocol exposes structured facts and readable output organizes them
Docnav raw protocol MUST 将机器可读事实表达为结构化 JSON 字段。Readable output MUST 把这些事实组织成面向阅读的 display text、summary 和布局，并且不成为 raw protocol fact source。

#### Scenario: 请求预算使用 canonical limit
- **WHEN** document operation request 携带分页预算
- **THEN** canonical protocol argument 是 `limit`
- **THEN** `limit` 是 positive integer，预算单位解释仍归 adapter 所有
- **THEN** schema、examples、typed arguments 和 operation handling 使用 `limit`

#### Scenario: cost 在协议中结构化，在 readable 输出中摘要化
- **WHEN** operation result 报告 cost
- **THEN** raw protocol 携带结构化 `cost.measurements[]`
- **THEN** 每个 measurement 包含机器可读的 `unit` 和 `value`
- **THEN** readable output 可以从这些 measurements 派生成紧凑成本摘要

#### Scenario: 导航条目分离 ref、事实字段和 display
- **WHEN** outline entries 或 find matches 返回
- **THEN** 每个 raw protocol item 保留 `ref` 作为 adapter-owned opaque string
- **THEN** label、location、summary、excerpt、rank、cost 和 metadata 等 item facts 在可用时使用结构化字段
- **THEN** readable output 拥有最终 display row

#### Scenario: info result 将 metadata 与摘要分离
- **WHEN** info 返回 document 或 adapter facts
- **THEN** raw protocol 使用结构化 document 和 adapter metadata 字段
- **THEN** readable output 可以把这些字段呈现为紧凑摘要

#### Scenario: error 投影保留结构化 details
- **WHEN** protocol output 返回 `ok: false`
- **THEN** `error.code` 选择已文档化的结构化 `error.details` shape
- **THEN** `error.message` 和 `error.guidance` 保持展示字段
- **AND WHEN** readable output 返回 failure
- **THEN** readable error projection 保留 primary diagnostic 的结构化 details

#### Scenario: continuation owner 保持稳定
- **WHEN** 引入 structured fields
- **THEN** `page` 仍是 protocol-owned next-page integer or null
