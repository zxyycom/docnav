本 change 目标是让 `docnav-adapter-sdk` direct CLI 和 adapter `invoke` 都消费共享标准参数能力；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local adapter-protocol delta。共享标准参数机制由 `docs/standard-parameters.md` 完整承接；`docs/adapter-contract.md` 只同步 SDK direct CLI 和 adapter `invoke` 的消费边界。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 标准参数必须由共享标准参数层驱动
`docnav-adapter-sdk` direct CLI 中来自 CLI flag、adapter direct CLI 配置文件或内置默认值的标准参数，MUST 使用 `args-config-parameters` 共享能力声明和解析。SDK direct CLI MUST 提供 SDK-owned 标准参数 registration set、adapter direct CLI 配置来源描述和 direct CLI 入口策略，并消费共享层返回的类型化标准参数、标准参数来源信息、配置源诊断和 operation argument binding metadata。跨 consumer 的标准参数 MUST 从共享 base definition 或 builder factory 派生，SDK registration MUST 引用共享 base identity。SDK direct CLI 的 config source reading、config mapping、argv parsing、help/default 文案、schema-backed validation、warning、类型化标准参数生成和 invoke request construction MUST 消费同一组 SDK 标准参数 registration、来源信息和共享配置读取结果；operation request construction 和最终 operation 参数生成由 SDK-owned operation builder 消费类型化标准参数完成。

#### Scenario: SDK registration 驱动 flag、配置和 help
- **WHEN** SDK 注册一个标准参数，例如 `defaults.output`
- **THEN** direct CLI flag 映射、配置映射、help/default 文案、invoke request argument construction 和 schema-backed validation 都引用该参数的 base definition 和 SDK registration
- **THEN** SDK registration 是该参数 flag/config/help/request binding 的唯一 SDK 映射来源

#### Scenario: SDK 复用跨 consumer base definition
- **WHEN** SDK 注册一个跨 consumer 标准参数，例如 `defaults.output`
- **THEN** SDK 从共享 base definition 或 builder factory 派生该 registration
- **THEN** SDK 只补充 SDK-owned registration、adapter 配置来源描述、CLI registration 或 operation registration
- **THEN** canonical key、value type、schema facet 和 validation semantics 来自共享 base definition

#### Scenario: SDK 定义驱动配置字段映射
- **WHEN** adapter direct CLI 配置文件包含一个已注册标准参数的 config path
- **THEN** SDK config mapping 使用对应标准参数 registration 把该字段映射为带来源的参数值
- **THEN** 后续参数处理链路按 base definition 的 schema facet 校验值，并按 `ParamKey<T>` 生成类型化标准参数
- **THEN** SDK-owned operation builder 消费类型化标准参数生成最终 operation 参数

#### Scenario: SDK 使用共享配置读取
- **WHEN** SDK direct CLI 提供 adapter 配置来源描述、项目级配置路径、用户级配置路径、显式覆盖路径和 SDK 标准参数 registration set
- **THEN** 共享标准参数层读取可用配置源、校验顶层 object，并按 config path 映射已注册字段
- **THEN** SDK 消费共享层返回的类型化标准参数、标准参数来源信息和配置源诊断
- **THEN** SDK 标准参数配置读取和 config path 映射的实现来源为共享层

#### Scenario: SDK 使用共享 binding 处理 request arguments
- **WHEN** SDK direct CLI 需要构造 protocol request
- **THEN** SDK 使用共享标准参数层返回的 operation argument binding metadata 序列化需要跨 protocol 传递的显式字段
- **THEN** SDK direct CLI 已解析出的配置值或默认值不因 request construction 被重新标记为 adapter `invoke` direct source

### Requirement: Adapter invoke 标准参数必须由共享标准参数层驱动
Adapter `invoke` 中来自 protocol request `arguments`、adapter invoke 入口配置源或内置默认值的标准参数，MUST 使用 `args-config-parameters` 共享能力声明和解析。SDK MUST 在识别 request envelope、operation 和 raw `arguments` 后，使用 operation argument binding metadata 把 request `arguments` 中的显式已映射标准参数作为 direct input；未映射字段按 adapter invoke 入口策略保留、丢弃或交给 adapter-owned validation。SDK 再按固定合并顺序合并 direct input、项目配置、用户配置和默认值，执行 schema-backed validation，并生成类型化标准参数、透传字段或等价 semantic request。Protocol request/result envelope shape 保持稳定；protocol request `arguments` 的标准参数字段、requiredness、schema 和 examples MUST 表达 adapter `invoke` 显式输入，而不是调用方已经完成配置/default 合并的最终参数。Adapter-owned `options` 或 native options 只有在对应 registration 或入口策略明确声明时才参与标准参数解析；否则按 adapter-owned policy 消费、丢弃或报错。

#### Scenario: Invoke request arguments 映射为 direct source
- **WHEN** SDK 收到可识别 operation 的 adapter `invoke` request
- **AND** request `arguments` 包含已注册标准参数 binding
- **THEN** SDK 使用共享 operation argument binding metadata 把显式 request arguments 映射为 direct source
- **THEN** 共享解析器使用 adapter invoke 入口策略生成类型化标准参数或等价 semantic request

#### Scenario: Invoke 未映射字段由 adapter policy 处理
- **WHEN** SDK 收到可识别 operation 的 adapter `invoke` request
- **AND** request `arguments` 包含没有 operation argument binding 的字段
- **THEN** protocol request schema 不因该字段本身失败
- **THEN** adapter invoke 入口策略决定保留、丢弃该字段或交给 adapter-owned validation

#### Scenario: Invoke 配置源参与标准参数合并
- **WHEN** adapter `invoke` request 省略一个可由配置或默认值提供的已注册标准参数
- **AND** adapter invoke 入口策略声明项目级或用户级配置 source provider
- **THEN** SDK 读取对应配置源并映射已注册字段
- **THEN** 共享解析器按固定合并顺序合并 direct input、project config、user config 和 default
- **THEN** SDK 不因 request 缺少该预解析字段而直接返回 protocol request schema 错误

#### Scenario: Invoke direct source 覆盖配置来源
- **WHEN** adapter `invoke` request arguments 显式提供 `limit_chars`
- **AND** 项目级 adapter 配置也提供对应标准参数值
- **THEN** direct input 按固定合并顺序覆盖项目/用户配置
- **THEN** adapter handler 消费合并校验后的类型化标准参数

## MODIFIED Requirements

### Requirement: 协议边界必须按当前契约硬校验
Docnav 协议与 adapter SDK MUST 使用当前 protocol、manifest 和 probe schema 以及语义校验判断输出是否符合当前契约。`protocol_version`、`manifest_version` 和 `probe_version` MUST 保留为固定 schema 识别字段，但 MUST NOT 参与 adapter 路由、安装、更新或 invoke 的版本区间协商。Protocol request schema MUST 使用 `args-config-parameters` 的 protocol request schema view 校验 request envelope、operation、document path、raw `arguments` object 和已出现已注册字段的基础 JSON 类型；标准参数字段是否缺失、默认值如何产出、配置来源如何补全、最终类型/范围/枚举是否合法，MUST 由共享标准参数解析器校验。未映射 request argument 字段由入口策略或 adapter-owned validation 处理。协议 schema 和 examples MUST 与该语义一致。

#### Scenario: 当前契约校验通过
- **WHEN** adapter manifest、probe 和 invoke 响应符合当前 schema
- **AND** 必需 envelope 字段、字段类型、operation/result shape 和语义校验全部通过
- **THEN** 协议层认为该 adapter 输出符合当前契约

#### Scenario: 当前契约校验失败
- **WHEN** adapter 输出缺少当前 schema 必需字段或字段类型不符
- **THEN** 校验失败原因包含字段或 schema 路径信息
- **THEN** 当前阶段失败
- **THEN** 未选中的 adapter 记录为候选失败证据，已选中的 adapter 返回稳定 adapter/protocol 错误

#### Scenario: 请求标准参数缺失交给解析器判断
- **WHEN** adapter `invoke` request envelope、operation 和 raw `arguments` object 符合 protocol request schema
- **AND** `arguments` 省略某个已注册标准参数字段
- **THEN** SDK 将可用 request arguments 作为 direct source
- **THEN** 共享解析器根据入口策略、固定合并顺序、配置来源和 default facet 决定最终值或返回标准参数 validation error

#### Scenario: 请求版本字段不匹配当前 schema
- **WHEN** invoke 请求中的 `protocol_version` 不是当前 schema 固定值
- **THEN** request schema 校验失败
- **THEN** SDK 返回 `INVALID_REQUEST`
- **THEN** SDK 不返回 `PROTOCOL_INCOMPATIBLE`

#### Scenario: 无法解析请求时使用当前协议识别字段
- **WHEN** SDK 无法解析 invoke stdin 为有效请求 envelope
- **THEN** failure response 的 `protocol_version` 使用当前 `PROTOCOL_VERSION` 常量
- **THEN** failure response 的 `request_id` 使用未知请求 id 占位
- **THEN** failure response 的 `operation` 为 `null`

## REMOVED Requirements

### Requirement: Adapter invoke 不读取 direct CLI 配置

**Migration**: Adapter `invoke` 改为声明 adapter invoke 入口策略，使用共享标准参数层处理 request arguments、项目/用户配置和默认值，并按固定合并顺序生成类型化标准参数。若某个 adapter-owned native option 仍要求只接受显式 request value，必须在对应 registration 或入口策略中表达，而不是通过 `invoke` 全局排除配置/default。
