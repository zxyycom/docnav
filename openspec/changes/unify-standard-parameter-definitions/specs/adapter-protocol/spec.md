本 change 目标是让 `docnav-adapter-sdk` direct CLI 和 adapter `invoke` 都消费共享 args/config 参数能力；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local adapter-protocol delta，主规范同步由 tasks 中的文档任务承接。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 标准参数必须由共享 args/config 参数层驱动
`docnav-adapter-sdk` direct CLI 中来自 CLI flag、adapter direct CLI 配置文件或内置默认值的标准参数，MUST 使用 `args-config-parameters` 共享能力声明和解析。SDK direct CLI MUST 提供 SDK-owned 标准参数 registration set、adapter direct CLI 配置域描述和 direct CLI source profile，并消费共享层返回的 standard parameter objects、typed standard params、标准参数来源信息、配置源诊断和 operation argument binding metadata。跨 consumer 的标准参数 MUST 从共享 base definition 或 builder factory 派生，SDK registration MUST 引用共享 base identity。SDK direct CLI 的 config source reading、config projection、standard parameter object merge、argv parsing、help/default 文案、schema-backed validation、warning、typed standard params 生成和 invoke request direct-source serialization MUST 消费同一组 SDK 标准参数 registration、standard parameter object projection、来源信息和共享配置读取结果；operation request construction 和最终 operation 参数生成由 SDK-owned operation builder 消费 typed standard params 完成。

#### Scenario: SDK registration 驱动 flag、配置和 help
- **WHEN** SDK 注册一个标准参数，例如 `defaults.output`
- **THEN** direct CLI flag 映射、配置投影、help/default 文案、invoke request argument construction 和 schema-backed validation 都引用该参数的 base definition 和 SDK registration
- **THEN** SDK registration 是该参数 flag/config/help/request binding 的唯一 SDK 映射来源

#### Scenario: SDK 复用跨 consumer base definition
- **WHEN** SDK 注册一个跨 consumer 标准参数，例如 `defaults.output`
- **THEN** SDK 从共享 base definition 或 builder factory 派生该 registration
- **THEN** SDK 只补充 SDK-owned registration、adapter 配置域描述、CLI surface 或 operation registration
- **THEN** canonical key、value type、schema facet 和 validation semantics 来自共享 base definition

#### Scenario: SDK 定义驱动配置字段投影
- **WHEN** adapter direct CLI 配置文件包含一个已注册标准参数的 config path
- **THEN** SDK config projection 使用对应标准参数 registration 把该字段投影为带来源的参数值
- **THEN** 后续参数处理链路按 base definition 的 schema facet 校验值，并按 `ParamKey<T>` 投影为 typed standard params
- **THEN** SDK-owned operation builder 消费 typed standard params 生成最终 operation 参数

#### Scenario: SDK 使用共享配置读取和投影
- **WHEN** SDK direct CLI 提供 adapter 配置域描述、项目级配置路径、用户级配置路径、显式覆盖路径和 SDK 标准参数 registration set
- **THEN** 共享 args/config 参数层读取可用配置源、校验顶层 object，并按 config path 把各配置源分别投影为 config standard parameter object
- **THEN** SDK 消费共享层返回的 config standard parameter objects、typed standard params、标准参数来源信息和配置源诊断
- **THEN** SDK 标准参数配置读取和 config path 投影的实现来源为共享层

#### Scenario: SDK 使用共享 binding 处理 request arguments
- **WHEN** SDK direct CLI 需要构造 protocol request
- **THEN** SDK 使用共享 args/config 参数层返回的 operation argument binding metadata 序列化需要跨 protocol 传递的 direct standard param source fields
- **THEN** SDK direct CLI 已解析出的配置值或默认值不因 request construction 被重新标记为 adapter `invoke` direct source

### Requirement: Adapter invoke 标准参数必须由共享 args/config 参数层驱动
Adapter `invoke` 中来自 protocol request `arguments`、adapter invoke source profile 配置源或内置默认值的标准参数，MUST 使用 `args-config-parameters` 共享能力声明和解析。SDK MUST 在识别 request envelope、operation 和 raw `arguments` 后，使用 operation argument binding metadata 把 request `arguments` 中的显式标准参数投影为 direct input standard parameter object；再按 adapter invoke source profile 读取项目/用户配置并投影为 config standard parameter objects，把默认值投影为 default standard parameter object，按统一全局来源优先级合并这些对象，执行 schema-backed validation，并生成 typed standard params 或等价 semantic request。Protocol request/result envelope shape 保持稳定；protocol request `arguments` 的标准参数字段、requiredness、schema 和 examples MUST 表达 resolver direct input source，而不是调用方已经完成配置/default 合并的最终参数。Adapter-owned `options` 或 native options 只有在对应 registration/source profile 明确声明时才参与标准参数解析；否则仍是 adapter-owned explicit arguments。

#### Scenario: Invoke request arguments 投影为 direct source
- **WHEN** SDK 收到可识别 operation 的 adapter `invoke` request
- **AND** request `arguments` 包含已注册标准参数 binding
- **THEN** SDK 使用共享 operation argument binding metadata 把显式 request arguments 投影为 direct standard param source
- **THEN** 共享 resolver 使用 adapter invoke source profile 生成 typed standard params 或等价 semantic request

#### Scenario: Invoke 配置源参与标准参数合并
- **WHEN** adapter `invoke` request 省略一个可由配置或默认值提供的已注册标准参数
- **AND** adapter invoke source profile 声明项目级或用户级配置 source provider
- **THEN** SDK 读取对应配置源并投影为 config standard parameter object
- **THEN** 共享 resolver 按统一全局来源优先级合并 direct input、project config、user config 和 default standard parameter objects
- **THEN** SDK 不因 request 缺少该预解析字段而直接返回 protocol request schema 错误

#### Scenario: Invoke direct source 覆盖配置来源
- **WHEN** adapter `invoke` request arguments 显式提供 `limit_chars`
- **AND** 项目级 adapter 配置也提供对应标准参数值
- **THEN** direct input standard parameter object 按统一全局来源优先级覆盖项目/用户配置标准参数对象
- **THEN** adapter handler 消费合并校验后的 typed standard params

## MODIFIED Requirements

### Requirement: 协议边界必须按当前契约硬校验
Docnav 协议与 adapter SDK MUST 使用当前 protocol、manifest 和 probe schema 以及语义校验判断输出是否符合当前契约。`protocol_version`、`manifest_version` 和 `probe_version` MUST 保留为固定 schema 识别字段，但 MUST NOT 参与 adapter 路由、安装、更新或 invoke 的版本区间协商。Protocol request schema MUST 使用 `args-config-parameters` 的 protocol request schema view 校验 request envelope、operation、document path、raw `arguments` object、已出现字段的基础 JSON 类型和字段可识别性；标准参数字段是否缺失、默认值如何产出、配置来源如何补全、最终类型/范围/枚举是否合法，MUST 在 request arguments 投影为 direct standard param source 后由共享 args/config 参数 resolver 校验。协议 schema 和 examples MUST 与该投影语义一致。

#### Scenario: 当前契约校验通过
- **WHEN** adapter manifest、probe 和 invoke 响应符合当前 schema
- **AND** 必需 envelope 字段、字段类型、operation/result shape 和语义校验全部通过
- **THEN** 协议层认为该 adapter 输出符合当前契约

#### Scenario: 当前契约校验失败
- **WHEN** adapter 输出缺少当前 schema 必需字段或字段类型不符
- **THEN** 校验失败原因包含字段或 schema 路径信息
- **THEN** 当前阶段失败
- **THEN** 未选中的 adapter 记录为候选失败证据，已选中的 adapter 返回稳定 adapter/protocol 错误

#### Scenario: 请求标准参数缺失交给 resolver 判断
- **WHEN** adapter `invoke` request envelope、operation 和 raw `arguments` object 符合 protocol request schema
- **AND** `arguments` 省略某个已注册标准参数字段
- **THEN** SDK 将可用 request arguments 投影为 direct standard param source
- **THEN** 共享 resolver 根据 source profile、统一全局来源优先级、配置标准参数对象和 default facet 决定最终值或返回标准参数 validation error

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
**Reason**: 本 change 将 CLI argv、MCP tool input 和 adapter `invoke` request arguments 统一为标准参数 direct input surface；`invoke` 继续维护“不读取配置、不补默认值”的独立行为会让标准参数解析、默认值、来源优先级和校验在入口之间漂移。

**Migration**: Adapter `invoke` 改为声明 adapter invoke source profile，使用共享 args/config 参数层把 request arguments、项目/用户配置和默认值分别投影为标准参数对象，并按统一全局来源优先级合并后生成 typed standard params。若某个 adapter-owned native option 仍要求只接受显式 request value，必须在对应 registration/source profile 中表达，而不是通过 `invoke` 全局排除配置/default。
