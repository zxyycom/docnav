本 docnav-contracts delta 定义 `docnav` 作为 core CLI router/manager 时，对 core release 内置 adapter-layer workspace crates、static adapter registry、adapter layer ownership 和默认执行边界的长期职责。

## MODIFIED Requirements

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST 负责项目根解析、核心配置、core release static adapter registry、adapter inspection、adapter 选择、adapter layer dispatch、协议字段校验、输出模式和错误映射。默认 document operation path MUST use adapter-layer workspace crates registered in the current core release as adapter implementations. Adapter layer ownership MUST remain a code and contract boundary rather than a separate default distribution boundary. Independent adapter packages、external adapter executables and command paths MUST NOT become default document operation implementation sources.

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 根据 path、配置、core release static adapter registry metadata、扩展名和 adapter-owned support check 选择 adapter
- **THEN** `docnav` 将 page 和 limit 等 core 通用参数写入显式 operation input
- **THEN** `docnav` 不从 adapter metadata、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: core release static adapter registry inspection
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** `docnav` 输出当前 core release static adapter registry 中 adapter library 的身份、版本、支持格式和 capabilities

#### Scenario: dynamic adapter management commands are not default surface
- **WHEN** 调用方执行 `docnav adapter install <source>`
- **OR** 调用方执行 `docnav adapter register <source>`
- **OR** 调用方执行 `docnav adapter update <adapter-id>`
- **OR** 调用方执行 `docnav adapter remove <adapter-id>`
- **THEN** `docnav` 不把这些命令作为有效默认 adapter management surface
- **THEN** 这些命令不会改变当前 release 的 static adapter registry

#### Scenario: External adapter artifact is not a default implementation source
- **WHEN** 项目配置、用户配置或历史 adapter record 指向 external adapter executable
- **AND** 调用方执行 document operation
- **THEN** `docnav` 不把该 executable 当作 adapter implementation source
- **THEN** adapter selection 只使用当前 core release static adapter registry 和当前请求输入

### Requirement: Adapter 选择按阶段校验
`docnav` MUST 先按显式 adapter、格式或 content type 提示校验 core release static adapter registry 中的 adapter；失败后 MUST 按扩展名匹配 registry candidate 并校验；仍失败时 MUST 逐个执行 adapter-owned support check，直到成功或全部失败。Selection MUST be based on current core release static adapter registry metadata、explicit request input and adapter-owned support checks. `docnav` MUST NOT 只凭格式提示或扩展名静默选择 adapter.

#### Scenario: 显式格式优先
- **WHEN** 调用方提供 `--format markdown`
- **THEN** `docnav` 优先使用 core release static adapter registry metadata 中匹配 `formats[].id` 的 adapter 候选并执行校验
- **THEN** 校验失败时继续扩展名匹配阶段

#### Scenario: 显式 content type 优先
- **WHEN** 调用方提供 `--format text/markdown`
- **THEN** `docnav` 优先使用 core release static adapter registry metadata 中匹配 `formats[].content_types[]` 的 adapter 候选并执行校验
- **THEN** 校验成功时选中该 adapter

#### Scenario: 扩展名和全量 support check
- **WHEN** 显式格式阶段未选中 adapter
- **THEN** `docnav` 按 core release static adapter registry metadata 的 `formats[].extensions[]` 匹配 path 扩展名并校验候选 adapter
- **THEN** 扩展名候选都失败时，`docnav` 逐个执行 adapter-owned support check
- **THEN** 全部阶段失败时返回 `FORMAT_UNKNOWN`

#### Scenario: Historical adapter artifact records do not provide candidates
- **WHEN** 项目或用户配置中存在历史 adapter artifact records
- **AND** 调用方执行 document operation
- **THEN** adapter 选择只使用当前 core release static adapter registry 和当前请求的显式输入
