本 spec delta 定义 `adopt-strict-input-boundaries` 对 `adapter-protocol` 的目标变更：在当前 core-linked adapter 架构下，公共输入由 core CLI、protocol request、config source 和 navigation input resolution 严格处理；linked adapter handler 接收已准备的 typed operation input。

## ADDED Requirements

### Requirement: Core-linked adapter input boundary 必须接收已准备的输入
Docnav core 和 navigation layer MUST 在 dispatch linked adapter handler 前完成 public input boundary 处理。Core MUST classify commands and pass config source descriptors/paths; `docnav-navigation` MUST load raw config sources and construct typed operation input from raw command, protocol request arguments, project/user config and built-in defaults, preserving declared adapter-owned native option source metadata. Linked adapter handlers MUST NOT read CLI argv、stdin、stdout、stderr、process cwd or process exit code to obtain operation input.

Invalid public input MUST fail before linked adapter business execution when it belongs to core CLI parsing、protocol envelope/request shape、config source loading、navigation input resolution mapping or operation applicability。Declared adapter-owned native options MAY be handed to the selected adapter through source-level static native option registry metadata；unsupported option、type mismatch or range invalid MUST be reported by selected adapter typed-field validation before format business handling continues.

#### Scenario: core CLI unknown argv 被拒绝在 adapter dispatch 前
- **WHEN** caller executes `docnav outline docs/guide.md --unknown --output readable-json`
- **THEN** core CLI returns an input diagnostic
- **THEN** navigation does not dispatch the linked adapter handler
- **THEN** failure output projects one primary `DiagnosticRecord`

#### Scenario: protocol request shape failure 停在 protocol owner
- **WHEN** a protocol request JSON value contains unknown envelope fields、missing required fields or malformed request shape
- **THEN** protocol input validation rejects the request at the protocol boundary
- **THEN** navigation input resolution does not receive the invalid envelope
- **THEN** failure output uses the protocol failure projection for the primary `DiagnosticRecord`

#### Scenario: known operation arguments 进入 navigation input resolution
- **WHEN** a protocol request envelope is valid but operation arguments contain wrong type、unmapped arguments or invalid values
- **THEN** navigation input resolution and typed-field processing produce validation diagnostics
- **THEN** linked adapter business handling does not execute
- **THEN** the owning surface projects the diagnostics as a failed document request

#### Scenario: declared native option handoff 保留 owner metadata
- **WHEN** CLI、config or protocol arguments provide `options.max_heading_level: 2`
- **AND** the source-level static native option registry declares the Markdown option source
- **THEN** navigation input resolution preserves source kind、owner、namespace、key and type variant metadata
- **THEN** the linked Markdown handler receives the merged native option value in prepared operation input

#### Scenario: selected adapter typed-field native option validation 返回结构化诊断
- **WHEN** adapter selection succeeds and prepared input contains an unsupported option、type mismatch or range invalid value for the selected adapter
- **THEN** selected adapter typed-field validation returns a structured diagnostic before handler execution
- **THEN** core/output projects that diagnostic through the selected raw or readable failure surface

## MODIFIED Requirements

### Requirement: Protocol 和 shared helpers 必须保持 linked adapter 边界
`docnav-protocol`、`docnav-json-io`、`docnav-adapter-contracts` and shared diagnostics/output helpers MAY be reused only when they preserve the current protocol、core CLI、config and linked adapter library ownership boundaries. Shared helpers MUST keep current document operation execution on core-owned source handling and linked adapter handler dispatch.

Protocol envelope、manifest/probe-shaped metadata and readable/protocol response serialization MUST stay at their owning boundaries. Valid operation arguments pass through navigation input resolution and typed-field processing before linked dispatch. Linked adapter handlers receive prepared input and return structured results or diagnostics; final raw/readable projection and process exit behavior remain owned by core/output surfaces.

#### Scenario: Protocol helper 区分 envelope 和 arguments 校验
- **WHEN** shared code decodes protocol request、protocol response、manifest or probe JSON value
- **THEN** protocol envelope、manifest and probe shape failures stay at the owning protocol or adapter metadata boundary
- **THEN** valid request envelopes pass known operation arguments to navigation input resolution and typed-field processing
- **THEN** typed-field metadata、navigation input mapping and owner semantic rules produce validation diagnostics for known operation arguments

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

### Requirement: Navigation config loading turns descriptors into source values
Docnav core MUST hand config source descriptors/paths to `docnav-navigation` before dispatching a navigation command. `docnav-navigation` MUST own raw config source loading, source priority and strict config input diagnostics for navigation commands. Missing default config paths MUST 表示 absent。Explicit config source paths 按 strict config input 处理：missing、unreadable、non-file、invalid JSON 或 non-object JSON sources MUST 使命令失败。Adapter-owned `options` values MUST be handed to selected adapter typed-field validation as native option sources with metadata preserved.

#### Scenario: 使用默认配置来源
- **WHEN** 调用方执行 Markdown document operation 且未覆盖配置来源
- **THEN** Docnav 使用当前配置策略解析项目级和用户级来源
- **THEN** 缺失默认来源表示该层配置 source absent
- **THEN** adapter-owned `options` facts remain explicit navigation input resolution source values

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

### Requirement: Config loading 只产出 navigation source values
Config loading MUST support configured `defaults.pagination.enabled`、`defaults.pagination.limit`、`defaults.output` and adapter-owned `options` object for the `docnav` entrypoint. Navigation input resolution MUST merge CLI/protocol direct input、project config、user config and built-in defaults into navigation source values according to source priority. Present or explicitly selected config sources are strict config inputs：unreadable files、invalid JSON、non-object root values、unknown top-level fields and unknown `defaults` fields MUST produce config diagnostics.

`options` object 是 explicit adapter-owned native option source。Declared `options` keys MUST be handed to selected adapter typed-field validation with source metadata；undeclared keys MUST produce native option diagnostics.

#### Scenario: pagination defaults 字段投影为 navigation source values
- **WHEN** 配置文件包含 `defaults.pagination.limit: 6000`
- **AND** 配置文件包含 `defaults.output: "readable-view"`
- **THEN** navigation input resolution 将 `defaults.pagination.limit` 合并为 `limit` 参数来源
- **THEN** navigation input resolution 将 `defaults.output` 合并为 `output` 参数来源

#### Scenario: 配置 options 合并为 adapter-owned native option source
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** navigation input resolution 将 `options` object 合并为 adapter-owned native options 参数来源
- **THEN** native option registry、value handoff and operation applicability metadata are preserved for selected adapter typed-field validation/extraction

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

#### Scenario: 参数来源对象交给 navigation input resolution
- **WHEN** 项目级配置包含非法 `defaults.output`
- **AND** 调用方未显式传入 `--output`
- **THEN** navigation input resolution 将该值合并为 `output` 参数来源
- **THEN** navigation input/output typed validation returns an input diagnostic
- **THEN** linked adapter handler does not execute

## REMOVED Requirements

### Requirement: Adapter process CLI owns document input parsing
**Reason**: 当前默认契约使用 core CLI + core config + protocol request source handling + linked adapter library dispatch。Adapter library receives prepared operation input and does not own argv、stdin/stdout or process exit behavior.

**Migration**: Use core strict CLI/protocol/config source handling, navigation input resolution source objects, generic native option handoff and selected adapter typed-field diagnostics. Valid inputs share the core-linked document operation pipeline.
