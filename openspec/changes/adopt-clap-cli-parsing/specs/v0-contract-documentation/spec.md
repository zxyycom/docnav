范围说明：本 delta 更新 v0 contract 文档中的本 change 覆盖 Rust CLI 解析、canonical document operation input 或等价 semantic request、strict invoke 边界、MCP ownership 和 readable warning envelope。

## ADDED Requirements

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
