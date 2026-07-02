本 spec delta 定义 `adopt-strict-input-boundaries` 对 `standard-parameter-resolution` 的目标变更：让标准参数来源解析服务严格公共输入边界，并把 adapter native options 建模为明确 owner 的输入源。

## ADDED Requirements

### Requirement: Adapter native options 必须是 explicit owner-scoped input sources
Adapter native options MUST 表达为 explicit owner-scoped input sources。Entry owner MUST 知道哪些 source locations 可以包含 adapter-owned options，并 MUST 将这些值交给 adapter/native option owner 校验或拒绝。

Unknown direct input、unknown config fields 和 undeclared native options 默认 MUST 产生 blocking diagnostics。只有 adapter/native option owner 声明 option namespace 并拥有校验时，native option value MAY 留在 standard parameter typed-field validation 之外。

#### Scenario: 已声明 native option 进入 owner 校验
- **WHEN** core CLI、config 或 protocol request input 包含已声明的 adapter native option
- **THEN** source construction 将其记录为 adapter-owned native option source
- **THEN** standard parameter resolution 记录该值，供 adapter/native option owner 校验
- **THEN** adapter/native option owner 在 handler execution 前校验或拒绝该 option

#### Scenario: 未声明 native option 返回输入诊断
- **WHEN** core CLI、config 或 protocol request input 包含 undeclared native option
- **THEN** source construction 或 native option owner 返回 input diagnostic
- **THEN** request 在 handler execution 前返回

## MODIFIED Requirements

### Requirement: Source construction 使用 pipeline-derived catalog/index
标准参数 source layer MUST 从 pipeline-derived catalog/index 构造 direct input、project config、user config 和 default sources，然后进入 resolution。Mapped values MUST continue to use typed-field metadata for identity, type and constraint validation.

Unmapped direct input MUST 产生 source-scoped unmapped-input diagnostics。Explicit adapter-owned native option source 中的输入交由 adapter/native option owner 校验或拒绝。Config input 对 unknown 或 unmapped fields 使用同一规则；adapter-owned `options` 内的字段由 native option owner 处理。

#### Scenario: Direct input 进入 source construction
- **WHEN** caller 收到 CLI argv tokens 或 decoded protocol request arguments JSON value
- **THEN** caller 将 direct input 直接传给 standard parameter source construction
- **THEN** direct processing paths 和 typed-field metadata 产出 mapped values 与 validation diagnostic events
- **THEN** unmapped direct input produces source-scoped unmapped-input diagnostics
- **THEN** explicit adapter-owned native option source 中的输入交由 native option owner 校验或拒绝
- **THEN** entry owners 将这些 diagnostics 映射为各自 surface-specific input error 或 `INVALID_REQUEST`

#### Scenario: Config JSON 通过 derived config path 映射
- **WHEN** project 或 user config JSON object 在 derived config path 上包含值
- **THEN** source construction 将该值映射到 catalog/index 中的 standard parameter identity
- **THEN** source 记录 project config 或 user config 作为 source kind

#### Scenario: Config unknown field 产生诊断
- **WHEN** config JSON object 包含未映射到任何 derived standard parameter entry 的字段
- **AND** 该字段不位于 adapter-owned native option source
- **THEN** source construction 产生 source-scoped unmapped-input diagnostic
- **THEN** entry owner 将 diagnostic 映射为 config input failure

#### Scenario: 明确 adapter options 进入 owner 校验
- **WHEN** config JSON object 包含 adapter-owned `options` object
- **AND** adapter/native option owner declares that source location
- **THEN** source construction returns an adapter-owned native option source
- **THEN** adapter or entry owner validates or rejects the native option values

#### Scenario: Defaults 进入导航配置补全
- **WHEN** 字段存在 static default 或 caller-provided dynamic default
- **THEN** default handling 将该值作为 default source candidate
- **THEN** default 通过与其它 mapped source values 相同的 typed-field metadata validation

### Requirement: Config source loading returns source diagnostics and does not own output
标准参数 source layer MUST load configured project/user config sources and hand off diagnostic events；diagnostic formatting、output channel 和 exit behavior remain owned by the entry owner。

Missing default project 或 user config paths MUST 表示 absent。Explicit config override paths 若 missing、unreadable、not files、invalid JSON 或 non-object JSON，MUST 产生 blocking config-source diagnostics。Default config paths 一旦存在但无法读取或解析，也 MUST 产生 blocking config-source diagnostics，因为它们表示 declared project/user state invalid。

#### Scenario: Pipeline 拥有普通 config loading
- **WHEN** pipeline 收到 project/user config paths 或 source descriptors
- **THEN** 标准参数层读取 JSON，校验顶层 value 是 object，并构造 config source
- **THEN** pipeline 提供普通路径的 JSON loading 结果

#### Scenario: Missing default config source 视为 absent
- **WHEN** default project 或 user config path 不存在
- **THEN** config source 被视为 absent
- **THEN** source loading 返回 absent source state

#### Scenario: Explicit config source invalid 时失败
- **WHEN** explicit project 或 user config override 缺失、不可读、invalid JSON 或不是 JSON object
- **THEN** source loading 产生 blocking config-source diagnostic
- **THEN** 该 source 产生 config-source diagnostic
- **THEN** entry owner 将 diagnostic 映射为 input 或 config failure
- **THEN** entry owner 返回 config failure outcome

#### Scenario: Existing default config source invalid 时失败
- **WHEN** default project 或 user config path 存在
- **AND** 文件 unreadable、invalid JSON 或不是 JSON object
- **THEN** source loading 产生 blocking config-source diagnostic
- **THEN** entry owner 将 diagnostic 映射为 config failure
- **THEN** entry owner 返回 config failure outcome

## REMOVED Requirements

### Requirement: Standard parameter passthrough 保持 owner-scoped
**Reason**: 新契约只接受 mapped standard parameters 和 explicit adapter-owned native option sources；unmapped public input 由 source owner 产生 blocking diagnostic。

**Migration**: 使用 adapter native option source handling 表达 declared options。Adapter/native option owner 校验或拒绝 declared options；undeclared input 进入 blocking diagnostic。
