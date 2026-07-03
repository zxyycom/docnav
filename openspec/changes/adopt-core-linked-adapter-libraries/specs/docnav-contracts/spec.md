本 docnav-contracts delta 定义 `docnav` 作为 core CLI router/manager 时，对 core release 内置 adapter-layer workspace crates、static adapter registry、adapter layer ownership 和默认执行边界的长期职责。

## MODIFIED Requirements

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST 负责项目根解析、核心配置 source loading、core release static adapter registry、adapter inspection、协议字段校验、输出模式和错误映射。默认 document operation path MUST use adapter-layer workspace crates registered in the current core release as adapter implementations. Adapter layer ownership MUST remain a code and contract boundary rather than a separate default distribution boundary. Internal operation orchestration, adapter selection and navigation input resolution MUST be owned by `docnav-navigation`, while adapter layer interface definitions, static descriptors and shared contract types MUST be owned by `docnav-adapter-contracts`. `docnav-navigation` MUST prepare requests and dispatch operation handlers; it MUST NOT act as an adapter loader. Independent adapter packages、external adapter executables、command paths and historical adapter artifact records MUST NOT become default document operation implementation sources.

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 从当前 core release static adapter registry 选择 adapter implementation
- **THEN** `docnav` resolves `docs/guide.md` to an absolute path before navigation dispatch
- **THEN** `docnav` 将 page、limit 和 merged native options 等参数准备为 operation input
- **THEN** `docnav` 不从 adapter metadata、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: adapter contract owner remains smaller than operation orchestration
- **WHEN** adapter crate 接入默认 document operation path
- **THEN** adapter crate 依赖 `docnav-adapter-contracts` 暴露的 adapter layer interface definitions
- **THEN** `docnav-navigation` 负责组合 `outline/read/find/info` 流程
- **THEN** adapter crate 不需要依赖独立 runtime SDK、dynamic registration API 或 adapter direct CLI 才能参与默认 document operation

#### Scenario: adapter interface uses operation-handler granularity
- **WHEN** adapter crate 实现 adapter layer interface
- **THEN** adapter handle exposes static descriptor metadata, probe check, source-level native option registry entries and `outline/read/find/info` operation handlers through `docnav-adapter-contracts`
- **THEN** parser、ref、navigation、pagination 和 native option semantics remain adapter-owned inside those handlers
- **THEN** `docnav-navigation` dispatches the selected operation handler instead of composing adapter ref splitter、locator、format probe validation or parser/navigation primitives across the adapter/core boundary

#### Scenario: native option registry feeds adapter handoff
- **WHEN** the source-level native option registry includes the Markdown `options.max_heading_level` entry
- **AND** request or config sources provide `options.max_heading_level`
- **THEN** navigation input resolution merges the value with source and registry metadata
- **THEN** the linked Markdown handler receives the final option value and validates support, type and range semantics

#### Scenario: navigation layer is not an adapter loader
- **WHEN** `docnav-navigation` dispatches an operation
- **THEN** it receives a selected linked adapter handle from core registry/routing
- **THEN** it prepares the request and calls the operation handler
- **THEN** it does not load executables, resolve command paths, or mutate runtime adapter registration

#### Scenario: adapter diagnostic boundary excludes exit code API
- **WHEN** linked adapter handling fails
- **THEN** the adapter layer returns structured diagnostic facts
- **THEN** core/output owns protocol/readable projection and final process exit code
- **THEN** adapter contract does not expose exit-code return semantics

#### Scenario: core release static adapter registry inspection
- **WHEN** 调用方执行 `docnav adapter list`
- **THEN** `docnav` 输出当前 core release static adapter registry 中 adapter library 的身份、版本和支持格式

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
`docnav` MUST base adapter selection on the current core release static adapter registry and adapter probe results. Declared adapter id MUST come only from `--adapter` or the effective `defaults.adapter` config value. When declared adapter id exists, `docnav` MUST look up only the matching static registry entry and probe it; it MUST NOT fall back to automatic discovery or any external implementation source. When no declared adapter id exists, `docnav` MUST traverse the static registry in registry order, probe each adapter, and select the first `supported: true` result. Format facts、content type facts、path/config metadata and manifest metadata remain inspection or adapter-owned recognition inputs; runtime selection uses declared lookup or registry-order probe. `docnav` MUST NOT treat project/user historical adapter artifact records, installed packages, external executables or command paths as adapter candidates for the default document operation path.

#### Scenario: 声明式 adapter 只检查同名 registry entry
- **WHEN** 调用方提供 `--adapter markdown`
- **OR** effective config provides `defaults.adapter = "markdown"`
- **THEN** `docnav` looks up only the `markdown` entry in the current core release static adapter registry
- **THEN** `docnav` probes that adapter before selecting it
- **THEN** `docnav` does not fall back to automatic discovery, manifest metadata selection, external adapter loading or dynamic registration

#### Scenario: 未声明 adapter 时按 registry 顺序 probe
- **WHEN** 调用方没有传入 `--adapter`
- **AND** config does not provide `defaults.adapter`
- **THEN** `docnav` traverses the current core release static adapter registry in registry order
- **THEN** `docnav` selects the first adapter whose probe returns `supported: true`
- **THEN** extension、content type、path/config metadata and manifest metadata remain outside the runtime selection order
- **THEN** registry order and probe results determine the selected adapter

#### Scenario: Historical adapter artifact records do not provide candidates
- **WHEN** 项目或用户配置中存在历史 adapter artifact records
- **AND** 调用方执行 document operation
- **THEN** adapter selection only uses declared adapter lookup or static registry ordered probe over current core release adapter implementations
