本 change 目标是让 CLI argv、invoke request arguments 和 MCP tool input 都映射为共享标准化参数来源；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local docnav-contracts delta。共享标准参数机制由 `docs/standard-parameters.md` 完整承接；跨入口契约文档只同步消费边界和稳定映射关系。

## ADDED Requirements

### Requirement: Direct input 必须映射为标准化参数来源
当标准参数暴露到 CLI argv、protocol request arguments 或 MCP tool inputs 时，core、SDK 和 MCP bridge MUST 使用 `args-config-parameters` 提供的 registration、operation binding、MCP tool metadata 或由其生成/同步的 artifact。Core 和 SDK 构造 protocol request 时，MUST 以 operation argument binding metadata 作为 `arguments` 字段路径的唯一映射来源，并且只序列化需要跨 protocol 传递的显式字段和入口策略保留的透传字段；已解析出的配置值或默认值不得仅因 request construction 被重新标记为 direct source。Adapter `invoke` MUST 将 request `arguments` 作为本入口显式输入，再按固定合并顺序处理配置和默认值；未映射字段按 adapter invoke 入口策略保留、丢弃或交给 adapter-owned validation。MCP tool input metadata MUST 由 tool-level operation 映射、对应 operation 的 registration set、MCP metadata 和可选 CLI argv transport metadata 生成或同步；MCP bridge MAY 继续把 tool input 映射到 core `docnav` CLI argv，且 CLI argv spelling 只作为 transport metadata。

#### Scenario: Invoke request argument 由 operation binding 序列化
- **WHEN** core 或 SDK 已经得到某个 document operation 需要跨 protocol 传递的显式字段
- **THEN** request construction 使用共享 operation argument binding metadata 把这些字段序列化到 protocol request `arguments`
- **THEN** 入口策略明确保留的透传字段可以随同写入 request `arguments`
- **THEN** operation argument binding metadata 是标准参数 protocol argument path 的唯一映射来源

#### Scenario: Invoke request argument 映射为直接输入标准参数
- **WHEN** SDK 收到可识别 operation 和 raw `arguments` object 的 invoke request
- **AND** 该 request 的 `arguments` 字段对应已注册标准参数 binding
- **THEN** SDK 使用共享 operation argument binding metadata 把显式 request argument 映射为标准参数来源
- **THEN** 共享解析器按固定合并顺序合并 direct input、project config、user config 和 default
- **THEN** 后续 operation handler 消费类型化标准参数或等价 semantic request

#### Scenario: MCP tool input 由 tool -> operation 映射生成
- **WHEN** `docnav-mcp` 暴露 document tool input，例如 `document_read.limit_chars`
- **THEN** `document_read` 先映射到 read operation
- **THEN** 该 input 的 schema 和标准语义来自 read operation 的 registered parameter set、MCP tool metadata 或由其生成的打包 artifact
- **THEN** MCP bridge 使用 metadata 中的 schema、默认值、stable serialized param identity、direct input 映射和可选 core CLI argv spelling metadata
- **THEN** core CLI argv spelling 只作为当前 transport projection metadata，不作为 MCP tool input 语义来源
- **THEN** MCP bridge 将该 input 映射为标准参数来源
- **THEN** 当前 transport 可以继续把该来源映射到 core `docnav` CLI argv

#### Scenario: 多入口字段名称映射到同一标准语义
- **WHEN** 同一个标准参数在 CLI、protocol request 和 MCP tool 中使用各自入口 spelling，例如 `--limit-chars`、`limit_chars` 或 tool input path
- **THEN** CLI flag 通过 CLI registration 绑定到同一个 stable param identity
- **THEN** protocol request argument 通过 operation argument binding 绑定到同一个 stable param identity
- **THEN** MCP tool input 通过 tool-level operation 映射、operation registration set 和 MCP tool metadata 绑定到同一个 stable param identity
- **THEN** protocol request argument 和 MCP tool input 名称由 operation binding 与 tool mapping metadata 分别确定，但二者都映射为标准化参数来源
