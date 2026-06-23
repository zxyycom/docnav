本 change 目标是让 CLI argv、invoke request arguments 和 MCP tool input 都投影为共享标准化参数来源；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 docnav-contracts delta，主规范同步由 tasks 中的文档任务承接。

## ADDED Requirements

### Requirement: 直接输入 surface 必须先投影为标准化参数来源
当标准参数暴露到 CLI argv、protocol request arguments 或 MCP tool inputs 时，core、SDK 和 MCP bridge MUST 使用 `args-config-parameters` 提供的 registration、operation binding、MCP tool metadata 或由其生成/同步的 artifact。Core request construction 和 SDK direct CLI request construction MUST 以 operation argument binding metadata 作为 protocol request `arguments` 字段路径的映射来源，并只把需要跨 protocol 传递的 direct standard param source fields 序列化为 adapter `invoke` direct source；已解析出的配置值或默认值不得仅因 request construction 被重新标记为 direct source。Adapter `invoke` request arguments MUST 使用同一 operation argument binding 投影为 direct input standard parameter object，再按统一全局来源优先级与项目配置、用户配置和默认值标准参数对象合并并校验。MCP tool input metadata MUST 由 tool-level operation 映射、对应 operation 的 registration set 和 MCP/CLI surface metadata 生成或同步；MCP bridge MUST 将 tool input 投影为 direct input standard parameter object，当前 transport MAY 继续把该 source 映射到 core `docnav` CLI argv，且 CLI argv spelling 只作为 transport projection metadata。

#### Scenario: Invoke request argument 由 operation binding 序列化
- **WHEN** core 或 SDK 已经得到某个 document operation 需要跨 protocol 传递的 direct standard param source fields
- **THEN** request construction 使用共享 operation argument binding metadata 把这些 direct source fields 序列化到 protocol request `arguments`
- **THEN** operation argument binding metadata 是标准参数 protocol argument path 的唯一映射来源

#### Scenario: Invoke request argument 投影为直接输入标准参数
- **WHEN** SDK 收到可识别 operation 和 raw `arguments` object 的 invoke request
- **AND** 该 request 的 `arguments` 字段对应已注册标准参数 binding
- **THEN** SDK 使用共享 operation argument binding metadata 把显式 request argument 投影为 direct input standard parameter object
- **THEN** 共享 resolver 按统一全局来源优先级对 direct input、project config、user config 和 default standard parameter objects 执行合并、schema-backed validation 和 typed runtime value 生成
- **THEN** 后续 operation handler 消费 typed standard params 或等价 semantic request

#### Scenario: MCP tool input 由 tool -> operation 映射生成
- **WHEN** `docnav-mcp` 暴露 document tool input，例如 `document_read.limit_chars`
- **THEN** `document_read` 先映射到 read operation
- **THEN** 该 input 的 schema 和标准语义来自 read operation 的 registered parameter set、MCP tool metadata 或由其生成的打包 artifact
- **THEN** MCP bridge 使用 metadata 中的 schema、默认值、stable serialized param identity、direct source projection 和可选 core CLI argv spelling metadata
- **THEN** core CLI argv spelling 只作为当前 transport projection metadata，不作为 MCP tool input 语义来源
- **THEN** MCP bridge 将该 input 映射为 direct standard param source
- **THEN** 当前 transport 可以继续把该 source 映射到 core `docnav` CLI argv

#### Scenario: 多 surface 名称映射到同一标准语义
- **WHEN** 同一个标准参数在 CLI、protocol request 和 MCP tool 中使用各自 surface spelling，例如 `--limit-chars`、`limit_chars` 或 tool input path
- **THEN** CLI surface 通过 CLI registration 绑定到同一个 stable param identity
- **THEN** protocol request argument 通过 operation argument binding 绑定到同一个 stable param identity
- **THEN** MCP tool input 通过 tool-level operation 映射、operation registration set 和 MCP tool metadata 绑定到同一个 stable param identity
- **THEN** protocol request argument 和 MCP tool input 名称由 operation binding 与 tool mapping metadata 分别确定，但二者都投影为标准化参数来源
