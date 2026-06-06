# v0-contract-documentation Specification

## Purpose
TBD - created by archiving change define-docnav-v0-docs. Update Purpose after archive.
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
`docnav` MUST 负责项目根解析、核心配置、adapter 发现、安装、更新、移除、选择、invoke 启动、协议校验、输出模式和错误映射。

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 根据 path、配置、manifest、扩展名和 probe 选择 adapter
- **THEN** `docnav` 将 page、limit_chars 和 options 写入显式 invoke 请求
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: 正式安装 adapter
- **WHEN** 调用方执行 `docnav adapter install <source>`
- **THEN** `docnav` 只接受 GitHub 链接或本地可执行文件来源
- **THEN** `docnav` 解析安装来源并执行 adapter `manifest`
- **THEN** manifest 通过 schema 校验且协议范围兼容后才注册 adapter
- **THEN** 本地可执行文件来源必须记录 SHA-256 hash

#### Scenario: 正式更新 adapter
- **WHEN** 调用方执行 `docnav adapter update <adapter-id>`
- **THEN** `docnav` 使用已记录来源获取候选新版本
- **THEN** 新版本 manifest 和协议兼容校验通过后才替换旧版本
- **THEN** 校验失败时保留旧版本并返回结构化错误

#### Scenario: 本地 exe hash 失配
- **WHEN** 已注册的本地可执行文件 hash 与安装记录不一致
- **THEN** `docnav` 不得继续调用该 adapter
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

### Requirement: Ref 可路由且唯一
文档 MUST 定义 path 负责定位文档并用于 `docnav` 选择 adapter；ref MUST 只定位当前文档内部区域，并由 adapter 拥有和解析。

#### Scenario: 从 outline 执行 read
- **WHEN** 调用方取得 outline ref
- **THEN** ref 可原样放入 read
- **THEN** read 按 path 选择 adapter 并由 ref 唯一定位内部区域

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
- **WHEN** 后续实现者依据 v0 文档规划测试
- **THEN** 能识别 protocol、readable、配置隔离、`docnav` 路由和 MCP 映射测试

### Requirement: 文档阅读路径清晰
README MUST 作为日常入口导航，MUST 包含 v0 范围、术语、角色化阅读路径和规则 owner 索引；README MUST NOT 作为完整规范副本重复定义细则。
Schema 和示例 MUST 作为校验材料；OpenSpec change MUST 作为变更依据、验收和审计历史，不作为日常实现主入口。

#### Scenario: 新实现者选择阅读路径
- **WHEN** 新实现者打开 README
- **THEN** 能在 README 中按角色找到应阅读的主规范文档
- **THEN** 能区分主规范、校验材料、参考材料和变更历史

#### Scenario: 维护重复规则
- **WHEN** 某条关键规则需要修改
- **THEN** 实现者能在 README 的规则 owner 索引中定位主规范
- **THEN** 其它文档只引用或摘要该规则

### Requirement: Ref 文档必须描述 Markdown heading ref 新格式
Ref 文档和示例 MUST 把 Markdown heading ref 描述为 adapter 拥有的字符串，格式为 `L{line}:{path}` 或 `L{line}#{ordinal}:{path}`，其中 `path` 表示 heading breadcrumb。文档 MUST 说明 Markdown 解析器接受显式 `#1`，但 canonical 生成结果省略 `#1`；文档 MUST 继续把 `doc:full` 描述为全文 fallback ref，并明确它不属于 heading ref 格式。

#### Scenario: Ref 规范展示无重复 heading
- **WHEN** 读者打开 Ref 文档
- **THEN** 首个 occurrence 的 Markdown heading 示例使用 `L1:Guide` 和 `L5:Guide > Install` 形式的 ref
- **THEN** 示例说明 `Guide > Install` 是 heading breadcrumb

#### Scenario: Ref 规范展示重复 heading path
- **WHEN** 读者查看 Ref 文档中的重复 heading path 示例
- **THEN** 首个 occurrence 使用 `L1:Repeat` 和 `L5:Repeat > Child` 形式的 ref
- **THEN** 后续 occurrence 使用 `L9#2:Repeat` 和 `L13#2:Repeat > Child` 形式的 ref

#### Scenario: 文档示例不保留 legacy ordinal suffix
- **WHEN** 实现者更新本 change 涉及的 docs 和 examples
- **THEN** Ref 文档、Markdown 示例和 OpenSpec task/spec 文案只使用 canonical heading ref 示例或 `doc:full`
- **THEN** 文档示例不保留旧方括号 ordinal 后缀 marker

