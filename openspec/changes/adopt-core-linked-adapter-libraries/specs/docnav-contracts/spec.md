本 docnav-contracts delta 定义 `docnav` 作为 core CLI router/manager 时，对 core release 内置 adapter-layer workspace crates、static adapter registry、adapter layer ownership 和默认执行边界的长期职责。

## MODIFIED Requirements

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST 负责项目根解析、核心配置、core release static adapter registry、adapter inspection、adapter 选择、adapter layer dispatch、协议字段校验、输出模式和错误映射。默认 document operation path MUST use adapter-layer workspace crates registered in the current core release as adapter implementations. Adapter layer ownership MUST remain a code and contract boundary rather than a separate default distribution boundary. Internal operation orchestration MUST be owned by `docnav-navigation`, while adapter layer interface definitions and shared contract types MUST be owned by `docnav-adapter-contracts`. Independent adapter packages、external adapter executables、command paths and historical adapter artifact records MUST NOT become default document operation implementation sources.

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 从当前 core release static adapter registry 选择 adapter implementation
- **THEN** `docnav` 将 page 和 limit 等 core 通用参数写入显式 operation input
- **THEN** `docnav` 不从 adapter metadata、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: adapter contract owner remains smaller than operation orchestration
- **WHEN** adapter crate 接入默认 document operation path
- **THEN** adapter crate 依赖 `docnav-adapter-contracts` 暴露的 adapter layer interface definitions
- **THEN** `docnav-navigation` 负责组合 `outline/read/find/info` 流程
- **THEN** adapter crate 不需要依赖独立 runtime SDK、dynamic registration API 或 adapter direct CLI 才能参与默认 document operation

#### Scenario: adapter interface starts from format-owned building blocks
- **WHEN** adapter crate 实现 adapter layer interface
- **THEN** adapter interface SHOULD prefer ref splitter、locator、format support check 和 parser/navigation primitives
- **THEN** operation-level `outline/read/find/info` handlers MAY replace these primitives only after design/spec/tasks record that the primitive split creates implementation complexity without corresponding product benefit

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
- **THEN** adapter implementation 只来自当前 core release static adapter registry

### Requirement: Adapter selection source 必须受 core release registry 约束
`docnav` MUST base adapter selection on the current core release static adapter registry, explicit request input and adapter-owned support checks. Format hints、content type hints、path information and config MAY guide selection, but they MUST NOT provide adapter implementation. `docnav` MUST NOT treat project/user historical adapter artifact records, installed packages, external executables or command paths as adapter candidates for the default document operation path.

#### Scenario: 显式格式提示只选择 registry candidate
- **WHEN** 调用方提供 `--format markdown`
- **THEN** `docnav` MAY use that hint to select or prioritize candidates
- **THEN** every candidate implementation comes from the current core release static adapter registry

#### Scenario: adapter-owned support check 只验证 registry candidate
- **WHEN** `docnav` 需要通过 adapter-owned support check 判断目标文档是否可处理
- **THEN** `docnav` only invokes support checks on adapter implementations registered in the current core release static adapter registry

#### Scenario: Historical adapter artifact records do not provide candidates
- **WHEN** 项目或用户配置中存在历史 adapter artifact records
- **AND** 调用方执行 document operation
- **THEN** adapter selection 只使用当前 core release static adapter registry、当前请求输入和 adapter-owned support checks
