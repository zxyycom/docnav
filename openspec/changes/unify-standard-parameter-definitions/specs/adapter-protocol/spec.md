本 change 目标是统一 core `docnav` 和 `docnav-adapter-sdk` direct CLI 的标准参数定义机制；本文档只是 `openspec/changes/unify-standard-parameter-definitions/` 下的未审核临时 adapter-protocol delta，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 标准参数必须由共享定义模型驱动
`docnav-adapter-sdk` direct CLI 中可同时来自 CLI flag、adapter direct CLI 配置文件或内置默认值的标准参数 MUST 由共享 Rust 标准参数定义模型声明。每个 SDK direct CLI 标准参数定义 MUST 至少表达 canonical key、配置文件 path、可选 CLI flag、help/default 文案、value kind、typed parser 或 validator、operation applicability、source priority、default provider、finalization rule 和 schema metadata。SDK direct CLI 的 config projection、argv parsing、help/default 文案、typed validation、warning、operation request construction 和最终 operation 参数生成 MUST 消费同一组 SDK 标准参数定义。

#### Scenario: SDK 定义驱动 flag、配置和 help
- **WHEN** SDK 注册一个标准参数定义，例如 `defaults.output`
- **THEN** direct CLI flag 映射、配置投影、help/default 文案和 typed validation 都引用该定义
- **THEN** SDK 不为同一个参数维护独立的 flag/config/help/validation 映射表

#### Scenario: SDK 定义驱动配置字段投影
- **WHEN** adapter direct CLI 配置文件包含一个已注册标准参数的 config path
- **THEN** SDK config projection 使用对应标准参数定义把该字段投影为带来源的参数值
- **THEN** 后续参数处理链路按定义声明的 validation 和 source priority 生成最终 operation 参数

#### Scenario: SDK 定义可提供 adapter 配置 schema metadata
- **WHEN** SDK 标准参数定义声明 JSON 配置 path、value kind、enum 或数值范围
- **THEN** 该 metadata 可用于生成 adapter direct CLI 配置 schema 参考材料
- **THEN** direct CLI runtime 不要求先加载生成后的 schema 文件才能读取配置

### Requirement: 标准参数定义不得改变 invoke 协议边界
共享标准参数定义 metadata MUST NOT 写入 adapter `invoke` stdin JSON protocol request。Adapter `invoke` MUST 继续只接受 schema-valid protocol input，并 MUST NOT 读取项目级或用户级 adapter direct CLI 配置来补全缺失参数。标准参数定义只适用于 direct CLI argv/config/default 归一，以及 core 在启动 adapter invoke 前的参数显式化。

#### Scenario: Invoke 不接收 definition metadata
- **WHEN** adapter `invoke` 收到 schema-valid outline request
- **THEN** request arguments 中只包含 protocol 定义的 operation 参数
- **THEN** 标准参数 definition metadata 不会加入 request arguments

#### Scenario: Invoke 不从配置补参数
- **WHEN** adapter `invoke` 从 stdin 收到缺少 protocol 必需字段的 request
- **AND** adapter direct CLI 配置文件包含某个标准参数默认值
- **THEN** SDK 按 protocol request validation 返回 `INVALID_REQUEST`
- **THEN** SDK 不读取 direct CLI 配置来补全 protocol request

### Requirement: Adapter native options 仍由 native option 机制拥有
Adapter-owned `options` object MUST 对 core `docnav` 保持 opaque，并且 MUST NOT 被提升为共享标准参数。SDK direct CLI MAY 保留 adapter-specific flags 和 config values 的 native option specs；标准参数定义 MUST 只覆盖 output mode、configuration path controls 或等价 SDK-owned common behavior 这类跨 surface 标准参数。

#### Scenario: Native option 不是标准参数
- **WHEN** Markdown adapter 声明 `options.max_heading_level`
- **THEN** SDK native option handling 校验该 option 的类型、范围和 operation applicability
- **THEN** 标准参数定义模型不把 `options.max_heading_level` 注册为 core 或 SDK standard parameter
