本 change 新增共享标准参数能力；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local args-config-parameters delta。归档后，`docs/standard-parameters.md` 完整承接该能力的主规范；入口主规范只同步各自消费边界并链接该文档。

## ADDED Requirements

### Requirement: 共享标准参数层必须拥有标准参数 base definition 和 registration set
共享 Rust 标准参数层 MUST 提供 builder-style 标准参数 base definition model。每个 base definition MUST 表达 `ParamKey<T>`、canonical key、value type、default facet、必选 schema facet、基础 validator 和 schema metadata。共享层 MUST 支持 consumer、CLI command、operation 和 MCP tool mapping 的 registration set 或 metadata；registration set MUST 显式声明该入口暴露的 config path、CLI flag mapping、operation argument binding 或 MCP tool input mapping。Base definition MUST NOT 通过全局 `.applies_to` 隐式决定所有入口、operation 或 MCP tool 暴露范围。

共享层 MUST 支持可复用 base definition 或 builder factory，使跨 consumer 复用的 canonical key 从同一 base 派生。Definition build/register 阶段 MUST 生成不可变 definition/registration，并对必需槽位、schema、canonical key fingerprint、flag、config path、operation argument binding、静态默认值、flag argv facet 与 schema facet 兼容性执行结构校验。

#### Scenario: Base definition 暴露稳定语义槽位
- **WHEN** 调用方声明一个标准参数 base definition，例如 `defaults.output`
- **THEN** builder 或等价声明 API 能表达 `ParamKey<T>`、canonical key、value type、default facet、schema facet、基础 validator 和 schema metadata
- **THEN** build/register 阶段生成不可变 base definition

#### Scenario: Registration set 显式暴露入口字段映射
- **WHEN** core、SDK、某个 operation 或 MCP tool mapping 需要暴露一个标准参数
- **THEN** 调用方从 base definition 派生 registration
- **THEN** registration 或 metadata 明确声明 config path、CLI flag mapping、operation argument binding 或 MCP tool input mapping
- **THEN** 未出现在该 registration set 或 metadata 中的参数不属于该入口、operation 或 MCP tool 的可用参数

#### Scenario: 跨 consumer 参数复用 base definition
- **WHEN** core 和 SDK 都注册 `defaults.output`
- **THEN** 两边从同一个共享 base definition 或 builder factory 派生该参数
- **THEN** consumer 只补充 owner-specific registration、配置来源描述或 command/operation registration set
- **THEN** canonical key、value type、schema facet、default facet 和 validation semantics 来自共享 base definition

#### Scenario: Config path 只接受 typed path builder
- **WHEN** 调用方注册 config-backed 标准参数
- **THEN** config path 输入只来自 typed path builder
- **THEN** build/register 阶段接受 typed path builder 作为 config path 输入来源
- **THEN** typed path 能导出 path segments、显示用 dotted path 和 schema 生成位置

#### Scenario: Flag argv facet 与 schema facet 兼容
- **WHEN** 标准参数 registration 声明 `ArgFacet::flag()` 或等价 no-value flag
- **THEN** registration 的 schema facet 必须是 boolean schema
- **THEN** build/register 阶段校验 no-value flag facet 与 boolean schema 匹配

#### Scenario: 默认值先产出再校验
- **WHEN** 标准参数 base definition 声明静态默认值
- **THEN** build/register 阶段使用该定义的 schema facet 校验静态默认值
- **WHEN** 标准参数 base definition 声明动态默认值 provider
- **THEN** runtime 在 provider 产出具体默认值后使用同一 schema facet 校验该值

### Requirement: 共享标准参数层必须生成类型化运行值
共享 Rust 标准参数层 MUST 从标准参数来源解析出类型化运行值。CLI argv、adapter `invoke` request arguments、MCP tool input 或等价入口显式输入 MUST 作为 direct input；项目级配置、用户级配置或其它入口声明的配置来源 MUST 作为对应 config source；默认值 MUST 作为 default source。每个入口 MUST 声明入口策略，列出它拥有的 direct input mapping、配置 source provider、默认 provider 和透传策略；入口策略不声明独立优先级。共享层 MUST 使用固定合并顺序合并已映射的标准参数并解析最终值：direct input、project config、user config、default。调用方 MUST 能通过 `ParamKey<T>` 从 `ResolvedStandardParams` 或等价 typed object 中取得已通过 schema-backed validation 的 `T` 值。共享层 MUST 返回入口策略明确保留的透传字段，但不得对透传 value 执行标准参数 schema-backed validation。Core request construction、SDK operation builder、adapter invoke handler、MCP mapping、document context 输出和测试 MAY 复用同一 typed runtime object，不需要重新声明参数类型检查。

#### Scenario: 调用方按 ParamKey 取得 typed value
- **WHEN** 共享层解析出 `defaults.limit_chars`
- **THEN** 调用方能通过 typed `ParamKey<PositiveInteger>` 取得 `PositiveInteger`
- **THEN** 调用方不需要把 raw JSON value 或 raw argv string 再次解析为该类型

#### Scenario: Typed runtime values 记录来源
- **WHEN** 同一标准参数可来自显式 argv、MCP tool input、invoke request argument、项目配置、用户配置或内置默认值
- **AND** 调用方提供了该入口的入口策略
- **THEN** 共享层把入口显式输入、项目配置、用户配置和默认值作为独立来源
- **THEN** 共享层按固定合并顺序合并这些来源中的已映射标准参数并解析最终值：direct input、project config、user config、default
- **THEN** 共享层返回每个标准参数的最终 typed value、透传字段和来源信息

### Requirement: 共享标准参数层必须拥有 direct input binding registry
共享 Rust 标准参数层 MUST 支持 direct input binding registry，把 CLI argv、protocol request `arguments` typed path、MCP tool input path 或等价入口显式输入映射到标准参数 `ParamKey<T>`。CLI flag spelling 属于 CLI registration；protocol argument path 属于 operation argument binding；MCP tool input path 属于 MCP tool metadata。简单参数 MAY 复用 stable binding name 派生默认 CLI long flag、protocol argument 字段和 MCP tool input 字段；使用特殊字段名、嵌套路径、兼容 alias 或特定 operation/tool 暴露范围的参数 MUST 显式声明对应 typed binding。

#### Scenario: Direct input binding 映射到 ParamKey
- **WHEN** direct input 提供一个已注册 CLI flag、protocol argument path 或 MCP tool input path
- **THEN** binding registry 将该输入映射到对应 `ParamKey<T>`
- **THEN** 未注册的 raw key/path 不会被映射为标准参数

### Requirement: 共享标准参数层必须按入口策略处理未映射输入
共享 Rust 标准参数层 MUST 支持入口级透传策略。未映射输入是指在对应 CLI registration、typed config path、operation argument binding 或 MCP tool metadata 中没有映射到 `ParamKey<T>` 的 raw key/path/value。入口策略 MUST 能声明未映射输入是保留到透传字段集合、丢弃，还是交给调用方 owner-specific validation。Core 入口策略 SHOULD preserve 需要下游 adapter 或 bridge 继续处理的未映射输入；adapter direct CLI 和 adapter `invoke` 入口策略 MAY drop 未映射 native 输入，因为 adapter 是对应格式参数的最终消费者。

#### Scenario: Core 保留下游参数
- **WHEN** core 入口策略声明 preserve passthrough
- **AND** direct input 或配置源中存在未映射但需要下游处理的 raw path/key
- **THEN** 共享层把该 raw path/key/value 和来源记录到透传字段集合
- **THEN** core request construction 可以按入口策略将该 passthrough 写入下游 request

#### Scenario: Adapter 丢弃未映射 native 输入
- **WHEN** adapter direct CLI 或 adapter `invoke` 入口策略声明 drop passthrough
- **AND** 输入中存在未映射 native key
- **THEN** 共享层不把该 key 作为标准参数校验
- **THEN** 共享层可以不在结果中保留该 key

#### Scenario: 简单参数可复用 stable name 派生多个入口字段
- **WHEN** 标准参数 base definition 声明 stable name，例如 `limit_chars`
- **THEN** CLI registration 能派生默认 CLI long flag
- **THEN** operation registration 能派生默认 protocol request argument path
- **THEN** MCP tool metadata 能派生默认 tool input path
- **THEN** 三个派生结果分别属于 CLI registration、operation argument binding 和 MCP tool metadata

#### Scenario: 特殊参数显式声明 typed operation argument 路径
- **WHEN** 某个标准参数在 protocol 中使用特殊字段名、嵌套路径、alias 或特定 operation 暴露范围
- **THEN** operation registration 必须显式声明对应 typed operation argument path
- **THEN** build/register 阶段接受 typed operation argument path 作为 binding 输入来源

#### Scenario: Binding 唯一性被校验
- **WHEN** 两个标准参数在同一 operation 绑定到同一个 protocol argument path
- **THEN** build/register 阶段报告 operation argument binding 冲突
- **WHEN** 两个标准参数在同一 CLI registration set 声明同一个 CLI flag spelling
- **THEN** build/register 阶段报告 CLI flag spelling 冲突
- **WHEN** 两个标准参数在同一 MCP tool metadata 中声明同一个 tool input path
- **THEN** build/register 或 metadata generation 阶段报告 MCP tool input binding 冲突

#### Scenario: Protocol 和 MCP 名称独立于 CLI flag spelling
- **WHEN** 一个标准参数的 CLI flag spelling 发生显式覆盖或兼容 alias 变更
- **THEN** protocol request argument path 仍由 operation argument binding 的 typed path 决定
- **THEN** MCP tool input path 仍由 MCP tool mapping metadata 决定

### Requirement: 共享标准参数层必须生成 MCP tool metadata
共享 Rust 标准参数层 MUST 能基于 MCP tool -> operation 映射、operation registration set、MCP metadata 和可选 CLI argv transport metadata 生成 MCP tool metadata。该 metadata MUST 能表达 tool input path、stable serialized param identity、canonical key、value kind、direct input 映射、CLI argv spelling 或其它 transport projection metadata、schema facet metadata、default metadata 和 operation registration membership；CLI argv spelling 只作为当前 transport projection metadata，不是 MCP 标准参数语义来源。Rust consumer MAY 将 stable serialized param identity 解析回 `ParamKey<T>`。MCP bridge MUST 使用该 metadata 暴露 tool input schema，并将 tool input 映射为标准参数来源；当前 transport MAY 继续把该来源映射为 core `docnav` CLI argv。MCP bridge 在本 change 中 MUST NOT 承接 adapter invoke request construction。JS 获取该 metadata 的推荐形态是 Rust 生成 JSON artifact；runtime metadata export 或人工同步的等价方案也可接受，但人工同步 MUST 有映射测试或 artifact/schema diff 证明未漂移。

#### Scenario: MCP tool input 从 operation registration 生成
- **WHEN** MCP tool 声明 `document_read` 对应 read operation
- **THEN** MCP tool input metadata 使用 read operation 的 registered parameter set 和 MCP tool mapping 生成
- **THEN** metadata 包含 tool input path、stable serialized param identity、canonical key、value kind、schema facet metadata 和默认值 metadata

#### Scenario: MCP tool input 映射到标准化参数来源
- **WHEN** MCP client 传入 `document_read.limit_chars`
- **THEN** MCP bridge 使用 metadata 找到对应 stable serialized param identity 和 direct input 映射
- **THEN** MCP bridge 将该 input 映射为标准参数来源
- **THEN** 当前 transport 可以继续把该来源映射为 core `docnav` CLI 参数

### Requirement: 共享标准参数层必须读取配置并映射已注册字段
共享 Rust 标准参数层 MUST 按调用方提供的配置来源描述读取标准配置源，校验顶层 JSON object，并按 registration 的 typed config path 映射已注册字段。Project/user 配置与 direct/default 一样，都是独立来源，再在最终 typed value resolution 中按固定合并顺序合并已映射标准参数。该顺序固定为 direct input、project config、user config、default；调用方入口只声明可用 source provider、配置来源、transport metadata 和透传策略，base definition 只声明参数语义。

#### Scenario: 配置源映射为标准参数来源
- **WHEN** 调用方提供配置来源描述、项目级配置路径、用户级配置路径、显式覆盖路径和标准参数 registration set
- **THEN** 共享层读取可用配置源并校验顶层 object
- **THEN** 共享层按 registration 的 typed config path 映射项目配置和用户配置中的已注册字段
- **THEN** 共享层返回配置源诊断和已映射的标准参数来源

#### Scenario: 单个配置源按 config path 映射为标准参数
- **WHEN** 某个配置源 object 包含一个字段
- **AND** registration set 中存在带同一 typed config path 的标准参数
- **THEN** 共享层将该字段映射为对应配置来源的标准参数字段
- **WHEN** 标准参数 registration 声明 no-config
- **THEN** 该参数的运行时值只来自 direct input 或 default facet

### Requirement: Schema facet 必须作为基础 value validation 的共同来源
每个标准参数 base definition MUST 声明 schema facet。Schema facet MUST 表达 runtime 基础校验和 schema metadata 输出所需的 type、enum、range、description 等结构化约束；默认值 metadata 来自 default facet。配置文件值、flag argv facet 处理后的 CLI value、operation argument value、MCP tool input value 和动态默认值产出的 value MUST 使用同一 schema facet 或由它编译出的 validator 完成基础校验；静态默认值 MUST 在 build/register 阶段使用同一 schema facet 校验。Runtime 配置解析使用 definition 内的 schema facet 或由它编译出的 validator。

共享层 MUST 支持 entry-specific schema view。Protocol request schema view MUST 只表达 request envelope、operation、document path、raw `arguments` object 和已出现已注册标准参数字段的基础 JSON 类型；它 MUST NOT 把解析器拥有的 requiredness、default completion、range、enum 或未映射 argument 字段的透传/丢弃决策提前变成 protocol schema hard error。解析器 schema view MUST 使用 schema facet 完成最终 value validation。MCP tool schema view 和 config schema view MAY 从同一 schema facet 输出更完整的 enum、range、description、requiredness 和 default metadata。

#### Scenario: Runtime 复用 schema facet 校验输入值
- **WHEN** 标准参数 base definition 声明 enum 或数值范围
- **THEN** 配置文件值使用该 schema facet 或由它编译出的 validator 校验
- **THEN** flag argv facet 处理后的 CLI value 使用该 schema facet 或由它编译出的 validator 校验
- **THEN** operation argument value 使用该 schema facet 或由它编译出的 validator 校验
- **THEN** MCP tool input value 使用该 schema facet 或由它编译出的 validator 校验
- **THEN** 动态默认值产出的 value 使用该 schema facet 或由它编译出的 validator 校验

#### Scenario: Protocol request schema view 不提前执行解析器校验
- **WHEN** 标准参数 base definition 声明 enum、range 或 default facet
- **AND** protocol request `arguments` 省略该字段或传入基础 JSON 类型正确但需要解析器校验的值
- **THEN** protocol request schema view 只校验 envelope、operation、document path、raw `arguments` object 和已出现已注册字段的基础 JSON 类型
- **THEN** 该字段是否缺失、默认值如何补全、range/enum 是否有效由共享解析器返回 typed value 或标准参数 validation error

#### Scenario: Schema metadata 可生成配置和 tool schema 参考材料
- **WHEN** 标准参数 registration 声明 typed config path、schema facet、default facet 和 operation argument binding
- **THEN** 共享层能按 schema view 输出配置 schema、protocol request schema、operation argument binding 和派生 MCP tool input schema 所需的 path、type、enum、range、description 和 default metadata
- **THEN** 生成后的 schema/example 作为验证材料、编辑器提示或打包参考使用

### Requirement: 同名标准参数必须由共享 base definition 派生
当多个 consumer 注册同一个 canonical key 时，标准参数 base definition MUST 来自同一个共享 base definition 或 builder factory。Base definition MUST 固定该 key 的 value type、schema facet、default facet 和 validation semantics。Consumer MAY 拥有不同参数集合、配置来源描述、CLI registration、operation registration 和 owner-specific operation builder。Registry MUST validate 同名 canonical key 的 base identity/fingerprint。

#### Scenario: 同名 key 在 core 和 SDK 中复用 base definition
- **WHEN** core 和 SDK 都注册 `defaults.output`
- **THEN** 两边使用同一个共享 base definition 或 builder factory
- **THEN** 共享 base definition 提供 canonical key、output value set、value type、schema facet 和 validation semantics

#### Scenario: 同名 key base identity 被校验
- **WHEN** 两个 registration 声明同一个 canonical key
- **THEN** build/register 阶段比较两个 registration 的 base definition identity/fingerprint
- **THEN** 只有 base identity/fingerprint 匹配的 registration 可以进入同一 registry
