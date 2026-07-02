本 spec delta 定义 `adopt-strict-input-boundaries` 对 `adapter-protocol` 的目标变更：在当前 core-linked adapter 架构下，公共输入由 core CLI、protocol request、config source 和标准参数 owner 严格处理；linked adapter handler 接收已准备的 operation input，并负责 adapter-owned native option consumption validation。

## ADDED Requirements

### Requirement: Core-linked adapter input boundary 必须接收已准备的输入
Docnav core 和 navigation layer MUST 在 dispatch linked adapter handler 前完成 public input boundary 处理。Core MUST 从 core CLI argv、protocol request arguments、project/user config 和 built-in defaults 构造 typed operation input，并保留 declared adapter-owned native option source metadata。Linked adapter handlers MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code to obtain operation input.

Invalid public input MUST fail before linked adapter business execution when it belongs to core CLI parsing、protocol envelope/request shape、config source loading、standard parameter mapping or operation applicability。Declared adapter-owned native options MAY be handed to the selected adapter through source-level static native option registry metadata；unsupported option、type mismatch or range invalid MUST be reported by the consuming adapter as structured diagnostics before format business handling continues.

#### Scenario: core CLI unknown argv 被拒绝在 adapter dispatch 前
- **WHEN** caller executes `docnav outline docs/guide.md --unknown --output readable-json`
- **THEN** core CLI returns an input diagnostic
- **THEN** navigation does not dispatch the linked adapter handler
- **THEN** failure output projects one primary `DiagnosticRecord`

#### Scenario: protocol request shape failure 停在 protocol owner
- **WHEN** a protocol request JSON value contains unknown envelope fields、missing required fields or malformed request shape
- **THEN** protocol input validation rejects the request at the protocol boundary
- **THEN** standard parameter processing does not receive the invalid envelope
- **THEN** failure output uses the protocol failure projection for the primary `DiagnosticRecord`

#### Scenario: known operation arguments 进入标准参数校验
- **WHEN** a protocol request envelope is valid but operation arguments contain wrong type、unmapped arguments or invalid values
- **THEN** standard parameter and typed-field processing produce validation diagnostics
- **THEN** linked adapter business handling does not execute
- **THEN** the owning surface projects the diagnostics as a failed document request

#### Scenario: declared native option handoff 保留 owner metadata
- **WHEN** CLI、config or protocol arguments provide `options.max_heading_level: 2`
- **AND** the source-level static native option registry declares the Markdown option source
- **THEN** standard parameter resolution preserves source kind、owner、namespace、key and type variant metadata
- **THEN** the linked Markdown handler receives the merged native option value in prepared operation input

#### Scenario: adapter-side native option validation 返回结构化诊断
- **WHEN** adapter selection succeeds and prepared input contains an unsupported option、type mismatch or range invalid value for the selected adapter
- **THEN** the linked adapter returns an adapter-owned structured diagnostic
- **THEN** core/output projects that diagnostic through the selected raw or readable failure surface
- **THEN** the adapter does not expose a process exit-code API

## MODIFIED Requirements

### Requirement: Protocol 和 shared helpers 必须保持 linked adapter 边界
`docnav-protocol`、`docnav-json-io`、`docnav-adapter-contracts` and shared diagnostics/output helpers MAY be reused only when they preserve the current protocol、core CLI、config and linked adapter library ownership boundaries. Shared helpers MUST keep current document operation execution on core-owned source handling and linked adapter handler dispatch.

Protocol envelope、manifest/probe-shaped metadata and readable/protocol response serialization MUST stay at their owning boundaries. Valid operation arguments pass through standard parameter/typed-field processing before linked dispatch. Linked adapter handlers receive prepared input and return structured results or diagnostics; final raw/readable projection and process exit behavior remain owned by core/output surfaces.

#### Scenario: Protocol helper 区分 envelope 和 arguments 校验
- **WHEN** shared code decodes protocol request、protocol response、manifest or probe JSON value
- **THEN** protocol envelope、manifest and probe shape failures stay at the owning protocol or adapter metadata boundary
- **THEN** valid request envelopes pass known operation arguments to standard parameter/typed-field processing
- **THEN** typed-field metadata、standard parameter mapping and owner semantic rules produce validation diagnostics for known operation arguments

#### Scenario: linked adapter handler 不拥有进程边界
- **WHEN** a linked adapter operation succeeds or returns diagnostic failure outcome
- **THEN** the handler returns structured operation result or diagnostic facts to core/navigation
- **THEN** it does not read stdin/stdout/stderr or process argv
- **THEN** core/output chooses protocol-json、readable-json or readable-view projection

#### Scenario: metadata helper 不提供 implementation source
- **WHEN** adapter metadata is rendered or validated as manifest/probe-shaped data
- **THEN** fields express adapter identity、formats、extensions、content types and observable metadata
- **THEN** metadata does not provide command paths、external executables、protocol version ranges or default/native option values
- **THEN** current core-linked document operations use the static registry descriptor and handler binding as the implementation source

### Requirement: Core config loading hands native options to adapters
Docnav core config loading MUST own config source discovery, source priority and strict config input diagnostics before dispatching a linked adapter library. Missing default config paths MUST 表示 absent。Explicit config source paths 按 strict config input 处理：missing、unreadable、non-file、invalid JSON 或 non-object JSON sources MUST 使命令失败。Adapter-owned `options` values MUST be handed to the selected adapter as native options with source metadata preserved.

#### Scenario: 使用默认配置来源
- **WHEN** 调用方执行 Markdown document operation 且未覆盖配置来源
- **THEN** Docnav 使用当前配置策略解析项目级和用户级来源
- **THEN** 缺失默认来源表示该层配置 source absent
- **THEN** adapter-owned `options` facts remain explicit config loading input

#### Scenario: 默认配置来源缺失表示 absence
- **WHEN** 默认项目级或用户级配置来源不存在
- **AND** 调用方未传入对应 config source override
- **THEN** Docnav 将该 config source 记为 absent
- **THEN** 缺失默认来源表示该层配置 source absent

#### Scenario: 覆盖配置来源不可用时失败
- **WHEN** 调用方显式提供不可用的配置来源
- **THEN** Docnav 尝试读取覆盖后的配置来源
- **THEN** Docnav 返回配置输入错误
- **THEN** Docnav 返回前停在 config input boundary
- **THEN** 该显式配置 source 的诊断成为本次调用结果

#### Scenario: Help keeps config ownership on core
- **WHEN** 调用方执行 document operation help
- **THEN** help 输出由 CLI parser/help owner 生成
- **THEN** adapter native option help does not redefine core config source ownership

### Requirement: Core config loading 只产出标准参数来源对象
Core config loading MUST support configured `defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` and adapter-owned `options` object for the `docnav` entrypoint. Core MUST merge CLI/protocol direct input、project config、user config and built-in defaults into standard parameter source objects according to source priority. Present or explicitly selected config sources are strict config inputs：unreadable files、invalid JSON、non-object root values、unknown top-level fields and unknown `defaults` fields MUST produce config diagnostics.

`options` object 是 explicit adapter-owned native option source。Declared `options` keys MUST be handed to adapter/native option owner validation with source metadata；undeclared keys MUST produce native option diagnostics.

#### Scenario: pagination defaults 字段投影为标准参数来源
- **WHEN** 配置文件包含 `defaults.pagination.limit: 6000`
- **AND** 配置文件包含 `defaults.output: "readable-view"`
- **THEN** core 将 `defaults.pagination.limit` 合并为 `limit` 参数来源
- **THEN** core 将 `defaults.output` 合并为 `output` 参数来源

#### Scenario: 配置 options 合并为 adapter-owned native option source
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** core 将 `options` object 合并为 adapter-owned native options 参数来源
- **THEN** native option registry、value handoff and operation applicability metadata are preserved for adapter-side validation

#### Scenario: 配置读取层拒绝未知字段
- **WHEN** 配置文件包含未知顶层字段或未知 `defaults` 字段
- **THEN** core 返回配置输入错误
- **THEN** 诊断包含配置路径和字段路径
- **THEN** core 返回前停在 config input boundary

#### Scenario: 配置源内容无效时失败
- **WHEN** core 读取到不可读、JSON 语法无效或顶层不是 JSON object 的配置源
- **THEN** core 返回配置输入错误
- **THEN** 诊断 details 包含 source level、path origin、path and reason code facts when available
- **THEN** core 返回前停在 config input boundary

#### Scenario: 参数来源对象交给标准参数处理链路
- **WHEN** 项目级配置包含非法 `defaults.output`
- **AND** 调用方未显式传入 `--output`
- **THEN** core 将该值合并为 `output` 参数来源
- **THEN** standard parameter/output typed validation returns an input diagnostic
- **THEN** linked adapter handler does not execute

### Requirement: Legacy external invoke projections are compatibility-only
Historical external invoke error codes or examples MAY remain parseable only as deprecated compatibility projections. Current core-linked adapter operations MUST NOT use an external adapter executable、stdout/stderr response parsing or adapter process exit code as the default implementation source.

#### Scenario: Current linked operation fails
- **WHEN** a linked adapter operation fails
- **THEN** the adapter layer returns structured diagnostic facts
- **THEN** core/output chooses the protocol/readable error projection and process exit code
- **THEN** historical external invoke fields are not required to model the implementation mechanism

#### Scenario: Historical projection remains parseable
- **WHEN** a legacy compatibility artifact contains external invoke failure details
- **THEN** schema MAY allow legacy details such as `exit_code` or `stderr`
- **THEN** docs identify those details as historical compatibility, not the current recommended path

## REMOVED Requirements

### Requirement: Adapter process CLI owns document input parsing
**Reason**: 当前默认契约使用 core CLI + core config + protocol request source handling + linked adapter library dispatch。Adapter library receives prepared operation input and does not own argv、stdin/stdout or process exit behavior.

**Migration**: Use core strict CLI/protocol/config source handling, standard parameter source objects, generic native option handoff and adapter-side structured diagnostics. Valid inputs share the core-linked document operation pipeline.
