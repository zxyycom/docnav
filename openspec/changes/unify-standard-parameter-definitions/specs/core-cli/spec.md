本 change 目标是让 core `docnav` 消费共享 args/config 参数能力；本文档是 `openspec/changes/unify-standard-parameter-definitions/` 下的 change-local core-cli delta，主规范同步由 tasks 中的文档任务承接。

## ADDED Requirements

### Requirement: Core CLI 标准参数必须由共享 args/config 参数层驱动
`docnav` core CLI 中可同时来自 CLI flag、core 配置文件或内置默认值的标准参数 MUST 使用 `args-config-parameters` 共享能力声明和解析。Core MUST 提供 core-owned 标准参数 registration set 和 core 配置域描述，并消费共享层返回的 standard parameter objects、typed standard params、标准参数来源信息、配置源诊断和 operation argument binding metadata。跨 consumer 的标准参数 MUST 从共享 base definition 或 builder factory 派生，core registration MUST 引用共享 base identity。Core `docnav` 的 argv parsing、config supported key listing、`config get/set/unset/list`、document context 输出、help/default 文案、schema-backed validation、typed standard params 生成和 invoke request direct-source serialization MUST 消费同一组 core 标准参数 registration、standard parameter object projection、来源信息和共享配置读取结果；最终 operation 参数由 core-owned operation builder 消费 typed standard params 生成。

#### Scenario: Core registration 驱动同一参数的多个 surface
- **WHEN** core 注册一个标准参数，例如 `defaults.output`
- **THEN** CLI flag 映射、配置 key 支持、help/default 文案、`config list` 展示、invoke request argument binding 和 schema-backed validation 都引用该参数的 base definition 和 core registration
- **THEN** core registration 是该参数 flag/config/help/request binding 的唯一 core 映射来源

#### Scenario: Core 复用跨 consumer base definition
- **WHEN** core 注册一个跨 consumer 标准参数，例如 `defaults.output`
- **THEN** core 从共享 base definition 或 builder factory 派生该 registration
- **THEN** core 只补充 core-owned registration、配置域描述、CLI surface 或 operation registration
- **THEN** canonical key、value type、schema facet 和 validation semantics 来自共享 base definition

#### Scenario: Core 定义保留参数来源信息
- **WHEN** 调用方通过显式 argv、项目配置、用户配置或内置默认值提供同一个 core 标准参数
- **THEN** core 先把显式 argv 投影为 direct input standard parameter object
- **THEN** core 把项目配置、用户配置和默认值分别投影为 standard parameter object
- **THEN** 共享层按统一全局来源优先级合并这些对象并解析最终 typed value：direct input standard parameter object、project config standard parameter object、user config standard parameter object、default standard parameter object
- **THEN** document context 输出能展示最终值和来源

#### Scenario: Core 使用共享配置读取和投影
- **WHEN** core 提供 core 配置域描述、项目级配置路径、用户级配置路径和 core 标准参数 registration set
- **THEN** 共享 args/config 参数层读取可用配置源、校验顶层 object，并按 config path 把各配置源分别投影为 config standard parameter object
- **THEN** core 消费共享层返回的 config standard parameter objects、typed standard params、标准参数来源信息和配置源诊断
- **THEN** core 标准参数配置读取和 config path 投影的实现来源为共享层

#### Scenario: Core 生成 operation 参数
- **WHEN** 共享层返回 core typed standard params 和来源信息
- **THEN** core-owned operation builder 消费 typed standard params 生成最终 operation 参数
- **THEN** core 继续拥有 `config get/set/unset/list`、document context 输出和 exit behavior

#### Scenario: Core 使用共享 binding 序列化 invoke request direct source
- **WHEN** core 准备为 document operation 构造 adapter invoke request
- **THEN** core 使用共享 args/config 参数层返回的 operation argument binding metadata 把适用的 direct standard param source fields 序列化到 request `arguments`
- **THEN** operation argument binding metadata 是 protocol argument path 的唯一映射来源

### Requirement: Core invoke request construction 必须输出标准参数 direct source
`docnav` MUST 在启动 adapter invoke 前使用共享 args/config 参数 resolver 完整解析 core CLI argv、core 项目/用户配置和默认值，并可将 typed standard params 用于 core-owned document context、adapter selection、request planning 或其它 core-owned 数据处理。构造 invoke request 时，core MUST 使用 operation argument binding metadata 和来源追踪，将需要跨 protocol 传递的 direct standard param source fields 序列化到 invoke request `arguments`。这些写入的 `arguments` 是 adapter `invoke` 的 protocol direct input source；它们不再定义为所有入口都必须预先完成配置/default 合并后的最终参数。Core 已解析的配置值或默认值 MUST NOT 被重新标记为 adapter `invoke` 的 direct standard param source。Adapter `invoke` 作为独立入口继续按共享 resolver 和统一全局来源优先级重新处理 request arguments、项目配置、用户配置和默认值。Core `docnav` MUST NOT 合成 format-specific `options`；adapter manifest MUST NOT 作为 options 或 default-parameter source。

#### Scenario: 省略 page
- **WHEN** 调用方省略 page
- **THEN** core source profile MAY 为 core-owned behavior 解析出默认 `page: 1`
- **THEN** `docnav` 不把该默认值作为 adapter `invoke` direct source 写入 request `arguments`
- **THEN** adapter `invoke` resolver 根据 adapter invoke source profile、统一全局来源优先级和 default facet 产出最终 `page` 值或返回标准参数 validation error

#### Scenario: 不写入格式 options
- **WHEN** 选中 adapter manifest 通过当前 schema 校验
- **THEN** `docnav` 仍能解析 core 默认参数
- **THEN** invoke 请求不包含由 manifest、配置或隐式默认值合成的格式 options

#### Scenario: request arguments 作为 adapter invoke direct source
- **WHEN** `docnav` 已经把 direct standard param source fields 写入 invoke request `arguments`
- **THEN** adapter `invoke` 将这些 fields 视为 direct standard param source
- **THEN** adapter `invoke` 仍使用自己的 source profile 决定可用的项目/用户配置或默认 provider，并按统一全局来源优先级合并

#### Scenario: invoke 当前契约不一致
- **WHEN** 选中 adapter 的 invoke 输出字段缺失、类型不符、operation/result shape 不匹配或语义校验失败
- **THEN** `docnav` 返回 adapter/protocol 错误
- **THEN** `docnav` 不继续尝试其它 adapter

## REMOVED Requirements

### Requirement: invoke 请求必须包含最终显式参数
**Reason**: 本 change 将 protocol request `arguments` 中的标准参数字段定义为 adapter `invoke` 的 direct input source。继续要求 invoke request 包含所有最终 resolved 参数，会让 `invoke` 无法和 CLI/MCP 一样复用标准参数 source merge、default 和 validation pipeline。

**Migration**: Core `docnav` 继续先完整运行共享 resolver，完成 core-owned typed standard params 和正常数据处理；随后使用 operation argument binding 与来源追踪序列化需要跨 protocol 传递的 direct standard param source fields。Adapter `invoke` 再把这些 fields、项目/用户配置和默认值分别作为标准参数对象，并按统一全局来源优先级合并。Core 配置或默认值不得仅因已被 core resolver 解析过就变成 adapter `invoke` direct source。
