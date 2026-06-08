范围说明：本 delta 更新 v0 contract 文档中的 Rust CLI 解析、canonical document operation input、strict invoke 边界、MCP ownership 和 readable warning envelope。

## ADDED Requirements

### Requirement: Rust CLI 参数解析必须服务 AI 维护和一次成功调用
Rust CLI 参数解析文档 MUST 将 `clap` 定义为核心 `docnav`、adapter direct CLI 和后续 Rust CLI 入口的首选 argv 解析基础。

文档必须描述以下稳定规则：

- 核心 `docnav` document CLI argv、adapter direct CLI document argv 和 adapter `invoke` JSON 在各自传输层解析成功后映射为 canonical document operation input。
- 传输层解析成功后，文档操作行为由共享语义归一、容错校验和 operation 执行负责。
- 当前 operation 的必需语义输入存在且实际使用参数有效时，即使 argv 包含未知 flag、多余 positional 或当前 operation 不使用的参数，CLI argv 也应继续成功执行。
- 当前 operation 实际使用的参数保持严格。
- Adapter `invoke` 在进入 canonical document operation input 前拒绝 malformed JSON、未知字段、缺失字段和类型错误。
- `clap` 是 CLI argv 实现基础；它不改变 protocol 字段、ref ownership、adapter 格式解析 ownership 或 MCP 传输语义。
- `docnav-mcp` 将 MCP tool call 映射到核心 `docnav` CLI，并包装 MCP 输出；它不进入 adapter SDK 解析或 adapter `invoke`。
- Readable warning 使用稳定 warning envelope：稳定 `kind`、非空 `reason`、`ignored_tokens: string[]` 和可选 family-specific 字段。
- 当前 warning family marker 包括 `cli_argv_ignored` 和 `adapter_candidate_failure`。
- CLI argv warning 的 exact token 分组、`reason` 文案和 token 消费顺序不稳定。

#### Scenario: 入口解析分层但共享文档操作语义管道
- **WHEN** 核心 `docnav` document CLI argv、adapter direct CLI document argv 或 adapter `invoke` JSON 分别通过各自入口解析
- **AND** 传输层校验成功
- **THEN** 文档操作请求映射为 canonical document operation input
- **THEN** 默认值、必需语义检查、native options、request 构造和 operation handler 使用共享逻辑
- **THEN** 不因入口不同复制业务参数规则

#### Scenario: AI 调用包含未知参数但必需语义完整
- **WHEN** AI agent 调用 Rust CLI 时传入未知 flag、多余 positional 或当前 operation 不使用的参数
- **AND** path、ref、query、page、limit_chars、output 等当前 operation 实际需要的语义可被解析并通过校验
- **THEN** CLI 继续执行对应 operation
- **THEN** 未知或多余输入最多产生阅读层 warning 或 stderr 诊断
- **THEN** CLI argv warning 使用 `kind: "cli_argv_ignored"`
- **THEN** protocol envelope、MCP transport ownership 和 adapter ref 语义保持不变

#### Scenario: 当前 operation 使用的已知参数仍严格
- **WHEN** Rust CLI 收到当前 operation 实际使用的已知参数
- **AND** 该参数缺少必需值或值无法通过类型、枚举或范围校验
- **THEN** CLI 返回输入错误
- **THEN** CLI 不通过宽松 argv 策略静默替换为默认值

#### Scenario: 阅读层 warning envelope 保留
- **WHEN** CLI 阅读层输出承载 warning
- **THEN** readable-json 或 MCP structuredContent 中的 warning item 包含稳定 `kind`、非空 `reason`、`ignored_tokens` 数组和可选 family-specific 字段
- **THEN** CLI argv warning 使用 `kind: "cli_argv_ignored"`
- **THEN** adapter candidate warning 使用 `kind: "adapter_candidate_failure"` 并保留 candidate family 字段
- **THEN** 测试只要求 stable envelope、family-specific 字段和用户可理解诊断
- **THEN** 测试不要求 exact ignored-token 分组、`reason` 文案或 token 消费顺序

#### Scenario: Invoke 入口仍严格校验
- **WHEN** adapter `invoke` 从 stdin 收到包含未知字段、缺少必需字段或参数类型错误的 JSON request
- **THEN** invoke 按 protocol schema 返回结构化失败
- **THEN** 该请求不进入 canonical document operation input
- **THEN** CLI argv 容错规则不用于忽略该 JSON request 的错误字段

#### Scenario: 有效 invoke 请求共享 operation 处理
- **WHEN** adapter `invoke` 从 stdin 收到 schema-valid 的 outline/read/find/info request
- **THEN** request 被映射为 canonical document operation input 或等价 request 结构
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
- **THEN** `docs/adapter-contract.md` 描述 adapter direct CLI、invoke strict transport validation 和 canonical document operation input 共享边界
- **THEN** `docs/testing.md` 描述成功路径、必要失败、help 可用、共享语义归一、stable warning envelope 和 schema 边界验证
- **THEN** `docs/schemas/readable-common.schema.json`、MCP outputSchema 示例和 `docs/examples/json/mcp-*-tool.json` 描述 stable warning envelope、当前 warning family marker 和 family-specific 字段
- **THEN** `docs/protocol.md` 如涉及 warning 或 protocol-shaped stdout 边界，只说明 protocol response、manifest 和 probe 不承载 CLI warning fields
