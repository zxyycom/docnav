本 spec delta 定义 `adopt-strict-input-boundaries` 对 `navigation-input-resolution` 的目标变更：让 navigation input resolution 服务严格公共输入边界，并把 adapter native options 建模为 selected adapter typed-field declarations 声明的 owner-scoped 输入源。

## ADDED Requirements

### Requirement: Adapter native options 必须是 explicit owner-scoped input sources
Adapter native options MUST 表达为 explicit owner-scoped input sources. `docnav-navigation` MUST know which selected-adapter source locations can contain adapter-owned options, and MUST validate/extract those values through selected adapter typed-field declarations before handler execution.

Unknown direct input、unknown config fields 和 undeclared native options 默认 MUST 产生 blocking diagnostics。只有 selected adapter typed-field declarations 声明 option namespace 并拥有校验规则时，native option value MAY enter request construction.

#### Scenario: 已声明 native option 进入 selected adapter typed-field 校验
- **WHEN** core CLI、config 或 protocol request input 包含已声明的 adapter native option
- **THEN** navigation input resolution records it as an adapter-owned native option source
- **THEN** selected adapter typed-field validation/extraction validates or rejects that option before handler execution

#### Scenario: 未声明 native option 返回输入诊断
- **WHEN** core CLI、config 或 protocol request input 包含 undeclared native option
- **THEN** navigation input resolution returns input diagnostic
- **THEN** request 在 handler execution 前返回

## MODIFIED Requirements

### Requirement: Config source loading returns source diagnostics and does not own output
`docnav-navigation` MUST load configured project/user config sources from core-supplied descriptors/paths and hand off diagnostic events; diagnostic formatting、output channel 和 exit behavior remain owned by core/output surfaces.

Missing default project 或 user config paths MUST 表示 absent。Explicit config override paths 若 missing、unreadable、not files、invalid JSON 或 non-object JSON，MUST 产生 blocking config-source diagnostics。Default config paths 一旦存在但无法读取或解析，也 MUST 产生 blocking config-source diagnostics because they represent declared project/user state invalid.

#### Scenario: Missing default config source 视为 absent
- **WHEN** default project 或 user config path 不存在
- **THEN** config source 被视为 absent
- **THEN** source loading 返回 absent source state

#### Scenario: Explicit config source invalid 时失败
- **WHEN** explicit project 或 user config override 缺失、不可读、invalid JSON 或不是 JSON object
- **THEN** navigation input resolution produces blocking config-source diagnostic
- **THEN** core maps diagnostic to input or config failure
- **THEN** core returns config failure outcome

#### Scenario: Existing default config source invalid 时失败
- **WHEN** default project 或 user config path 存在
- **AND** 文件 unreadable、invalid JSON 或不是 JSON object
- **THEN** navigation input resolution produces blocking config-source diagnostic
- **THEN** core maps diagnostic to config failure
- **THEN** core returns config failure outcome
