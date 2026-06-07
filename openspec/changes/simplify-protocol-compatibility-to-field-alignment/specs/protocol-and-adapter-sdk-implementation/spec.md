## ADDED Requirements

### Requirement: 协议边界必须按当前契约硬校验
Docnav 协议与 adapter SDK MUST 使用当前 protocol、manifest 和 probe schema 以及语义校验判断输出是否符合当前契约。`protocol_version`、`manifest_version` 和 `probe_version` MUST 保留为固定 schema 识别字段，但 MUST NOT 参与 adapter 路由、安装、更新或 invoke 的版本区间协商。

#### Scenario: 当前契约校验通过
- **WHEN** adapter manifest、probe 和 invoke 响应符合当前 schema
- **AND** 必需字段、字段类型、operation/result shape 和语义校验全部通过
- **THEN** 协议层认为该 adapter 输出符合当前契约

#### Scenario: 当前契约校验失败
- **WHEN** adapter 输出缺少当前 schema 必需字段或字段类型不符
- **THEN** 校验失败原因包含字段或 schema 路径信息
- **THEN** 当前阶段失败
- **THEN** 未选中的 adapter 记录为候选失败证据，已选中的 adapter 返回稳定 adapter/protocol 错误

### Requirement: Manifest 必须只承载 adapter 能力声明
Adapter manifest MUST restrict its field ownership to adapter identity, supported formats, extensions, content types, and capabilities. Manifest schema MUST reject protocol range fields and `recommended_parameters`.

#### Scenario: 读取 manifest
- **WHEN** adapter 输出 manifest
- **THEN** manifest 字段集合只表达 adapter 身份、支持格式、扩展名、content type 和 capabilities
- **THEN** 格式专属默认值不通过 manifest 传给 `docnav`

#### Scenario: Manifest 包含旧字段
- **WHEN** adapter manifest 包含 `protocol.min`、`protocol.max` 或 `recommended_parameters`
- **THEN** manifest schema 校验失败
- **THEN** adapter 在当前阶段不可用

## REMOVED Requirements

### Requirement: 协议版本兼容必须按闭区间判断
**Reason**: v0 阶段不维护多协议版本共存；闭区间协商让 adapter 选择、错误边界和测试矩阵过重。当前可用性以 schema、字段类型、operation/result shape 和语义校验为准。

**Migration**: 保留 `protocol_version: "0.1"` 作为固定 schema 识别字段；移除 `ProtocolRange`、闭区间选择 API、manifest `protocol.min/max` 字段和 `PROTOCOL_INCOMPATIBLE` 稳定错误。
