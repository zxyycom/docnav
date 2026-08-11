## MODIFIED Requirements

### Requirement: `docnav` 是 core CLI router/manager
`docnav` MUST 负责项目根解析、核心配置、adapter 发现、安装、更新、移除、选择、invoke 启动、协议字段校验、输出模式和错误映射。

#### Scenario: 读取 Markdown outline
- **WHEN** 调用方执行 `docnav outline docs/guide.md`
- **THEN** `docnav` 根据 path、配置、manifest、扩展名和 probe 选择 adapter
- **THEN** `docnav` 将 page 和 limit_chars 等 core 通用参数写入显式 invoke 请求
- **THEN** `docnav` 不从 manifest、配置或隐式默认值生成格式专属 `options`
- **THEN** adapter 生成的 ref 和 display 被保留到阅读输出

#### Scenario: 正式安装 adapter
- **WHEN** 调用方执行 `docnav adapter install <source>`
- **THEN** `docnav` 只接受 GitHub 链接或本地可执行文件来源
- **THEN** `docnav` 解析安装来源并执行 adapter `manifest`
- **THEN** manifest 只声明 adapter 身份、支持格式、扩展名、content type 和 capabilities
- **THEN** manifest 通过当前 schema、必需字段、字段类型和语义校验后才注册 adapter
- **THEN** 本地可执行文件来源必须记录 SHA-256 hash

#### Scenario: 正式更新 adapter
- **WHEN** 调用方执行 `docnav adapter update <adapter-id>`
- **THEN** `docnav` 使用已记录来源获取候选新制品
- **THEN** 新制品 manifest 通过 schema、必需字段、字段类型和语义校验后才替换旧记录
- **THEN** 校验失败时保留旧记录并返回结构化错误

#### Scenario: 本地 exe hash 失配
- **WHEN** 已注册的本地可执行文件 hash 与安装记录不一致
- **THEN** `docnav` 在 invoke 前阻断该 adapter
- **THEN** `docnav` 返回 `ADAPTER_UNAVAILABLE` 且 `details.reason` 为 `hash_mismatch`
