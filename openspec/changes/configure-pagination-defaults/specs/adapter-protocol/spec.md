本 change 目标是将分页默认值统一收敛到 `defaults.pagination`，并让 adapter SDK direct CLI 通过标准参数定义映射配置和 CLI flag，在进入 operation handler 前初始化最终 `limit_chars`；本文档只是 `openspec/changes/configure-pagination-defaults/` 下的未审核临时 adapter-protocol delta，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 使用标准参数定义映射配置和 argv
`docnav-adapter-sdk` direct CLI MUST 为同时支持配置文件和 CLI flag 来源的标准参数提供 direct CLI 标准参数定义。每个定义 MUST 声明 canonical path、value kind/validation、argv flag/parser、config projection、operation applicability、default source、source priority、help/default metadata 和 finalization rule。SDK direct CLI 的 config projection、argv parsing、help/default text 和 typed validation MUST 消费这些定义。

`defaults.pagination.enabled`、`defaults.pagination.limit_chars` 和 `defaults.output` MUST 由 SDK direct CLI 标准参数定义表示。Adapter native options MUST 保持为 `options` object 下的 adapter-owned value，MUST NOT 提升为 core 标准参数。标准参数定义机制 MUST NOT 向 adapter invoke protocol request 添加字段。

#### Scenario: SDK 定义映射 pagination flag 和配置路径
- **WHEN** SDK 注册 `defaults.pagination.enabled` 标准参数定义
- **THEN** `--pagination enabled|disabled` 和配置 key `defaults.pagination.enabled` 被解析为同一标准参数的不同来源
- **THEN** direct CLI help/default text 和 typed validation 使用同一定义
- **THEN** operation request construction 只接收最终 operation argument values

#### Scenario: SDK 定义不改变 invoke 协议
- **WHEN** adapter `invoke` 收到 schema-valid request
- **THEN** SDK 直接校验 stdin protocol request
- **THEN** standard parameter definition metadata 不会加入 request arguments

### Requirement: Adapter SDK direct CLI 支持 pagination argv 覆盖
`docnav-adapter-sdk` direct CLI document commands MUST 为分页文档操作暴露 `--pagination enabled|disabled`。该 flag MUST 映射为标准参数 `defaults.pagination.enabled` 的显式来源，并且 MUST NOT 作为 `pagination` 字段写入 protocol request arguments。

`--limit-chars` MUST 映射为标准参数 `defaults.pagination.limit_chars` 的显式来源。`--pagination enabled` MUST 让本次调用启用分页，并使用显式 `--limit-chars` 或解析后的 pagination limit。`--pagination disabled` MUST 让本次调用不启用分页，并把最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值。`--pagination disabled` 与同一次调用中的 `--limit-chars` MUST 被接受，并 MUST 按“配置提供 `enabled: false` 和 `limit_chars`”相同的标准参数归一规则处理。

#### Scenario: Direct CLI disables pagination from argv
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --pagination disabled`
- **THEN** SDK 将最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值
- **THEN** operation request construction 不包含 `pagination` 字段

#### Scenario: Direct CLI enables pagination over disabled config
- **WHEN** 项目级 adapter 配置设置 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --pagination enabled`
- **THEN** SDK 启用本次调用的分页
- **THEN** SDK 使用生效 `defaults.pagination.limit_chars` 或 adapter 内置默认预算

#### Scenario: Direct CLI pagination disabled accepts explicit limit source
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --pagination disabled --limit-chars 120`
- **THEN** SDK 将 `defaults.pagination.enabled` 解析为显式 `false`
- **THEN** SDK 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** SDK 将最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值

## MODIFIED Requirements

### Requirement: Adapter SDK direct CLI 支持自身配置域
`docnav-adapter-sdk` direct CLI MUST 支持读取解析后的项目级和用户级 adapter 配置文件。Direct CLI document operation MUST 按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”的优先级合并参数来源，并 MUST 在进入 operation request construction 前合并为标准 direct CLI 参数来源对象。配置读取层 MUST 只从配置文件投影 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`、`defaults.output` 和完整 `options` object；native option key 注册、value 处理和 operation 适用性由后续 direct CLI 参数处理链路决定。`path`、`ref`、`query` MUST 来自 argv，`page` MUST 来自 argv 或入口固定默认 `1`。

Direct CLI 标准参数来源对象 MUST 从 SDK direct CLI 标准参数定义构造。新增 direct CLI 标准参数 MUST 只需要新增或更新对应定义和测试，并通过同一定义驱动 parser、config、help 和 context 映射。

`defaults.pagination.enabled` MUST 默认为 `true`。`defaults.pagination.limit_chars` MUST 是正整数，默认值由 SDK direct CLI 调用方提供并在 `docnav-markdown` 中保持为 `6000`。当最终 `defaults.pagination.enabled` 为 `false` 时，SDK MUST 在进入 operation request construction 或 operation handler 前将最终 adapter request `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值；该规则不受 `defaults.pagination.limit_chars` 来源影响。

#### Scenario: Direct CLI 使用项目级 pagination 配置
- **WHEN** 项目级 `.docnav/docnav-markdown.json` 设置 `defaults.pagination.enabled: true`
- **AND** 项目级 `.docnav/docnav-markdown.json` 设置 `defaults.pagination.limit_chars: 120`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md` 且未传入 `--limit-chars`
- **THEN** SDK 将项目级配置中的 pagination 合并到标准 direct CLI 参数来源对象
- **THEN** 最终 `limit_chars` 在进入 operation handler 或 request construction 前已经显式化为 `120`

#### Scenario: Direct CLI 配置关闭分页
- **WHEN** 项目级 adapter 配置设置 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown read docs/guide.md --ref H:L1:H1`
- **THEN** SDK 将最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值
- **THEN** adapter operation handler 不需要知道配置来源即可按该预算执行

#### Scenario: 显式 limit_chars 不隐式启用分页
- **WHEN** 项目级 adapter 配置设置 `defaults.pagination.enabled: false`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --limit-chars 120`
- **THEN** direct CLI 将 `defaults.pagination.limit_chars` 解析为显式 `120`
- **THEN** direct CLI 保持 `defaults.pagination.enabled: false`
- **THEN** SDK 将最终 adapter request 的 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值

#### Scenario: 用户级配置作为项目级缺省
- **WHEN** 默认用户配置目录下的 `docnav-markdown.json` 设置 `defaults.output`
- **AND** 项目级配置没有设置 `defaults.output`
- **AND** 调用方未传入 `--output`
- **THEN** direct CLI 使用用户级配置中的 output 默认值

#### Scenario: 配置合并后只暴露标准参数来源对象
- **WHEN** SDK 完成 argv、项目级配置、用户级配置和内置默认值合并
- **THEN** operation request construction 只消费标准 direct CLI 参数来源对象处理后的最终 operation 参数
- **THEN** operation handler 不需要知道配置文件路径、配置来源或合并细节
- **THEN** 配置文件中的字段不会生成或覆盖 `path`、`ref`、`query` 或 `page`

### Requirement: Adapter SDK direct CLI 配置只产出标准参数来源对象
Adapter direct CLI config MUST 支持通用 `defaults.pagination.enabled`、`defaults.pagination.limit_chars`、`defaults.output` 和 `options` object。SDK MUST 按优先级把 argv、项目级配置、用户级配置和内置默认值合并为标准 direct CLI 参数来源对象。配置合并阶段 MUST 只处理配置源读取、固定字段投影、来源优先级和配置源跳过 warning；合并结果 MUST 表示为标准 direct CLI 参数来源对象和 direct CLI warning。配置源根值 MUST 是 JSON object。配置读取层 MUST 将 `defaults.pagination` 投影为 pagination 参数来源、将 `defaults.output` 投影为 `output` 参数来源、将 `options` object 原样投影为 native options 参数来源。生成后的参数来源对象 MUST 交给既有 direct CLI 参数处理链路完成类型、范围、枚举、native option 注册、最终 `limit_chars` 初始化和 operation 适用性处理。

固定 `defaults.*` 字段投影 MUST 由 SDK direct CLI 标准参数定义驱动。`options` object 投影保持 native-option pass-through，不由标准参数定义机制解释。

#### Scenario: defaults 字段投影为标准参数来源
- **WHEN** 配置文件包含 `defaults.pagination.enabled: true`
- **AND** 配置文件包含 `defaults.pagination.limit_chars: 6000`
- **AND** 配置文件包含 `defaults.output: "readable-view"`
- **THEN** SDK 将 `defaults.pagination` 合并为 pagination 参数来源
- **THEN** SDK 将 `defaults.output` 合并为 `output` 参数来源

#### Scenario: disabled pagination 投影后由参数处理链路初始化预算
- **WHEN** 配置文件包含 `defaults.pagination.enabled: false`
- **THEN** 配置读取层只投影该 pagination 参数来源
- **THEN** 后续 direct CLI 参数处理链路将最终 `limit_chars` 初始化为协议 `PositiveInteger` 可表示的最大值

#### Scenario: 配置 options 合并为标准 native option 参数
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** SDK 将 `options` object 合并为标准 native options 参数来源
- **THEN** native option 注册、value 处理和 operation 适用性由既有 native option 处理链路决定

#### Scenario: 配置读取层不校验未知字段
- **WHEN** 配置文件包含未知顶层字段或未知 `defaults` 字段
- **THEN** 配置读取层不因该字段产生配置源 warning
- **THEN** 该字段不参与标准 direct CLI 参数来源对象投影
- **WHEN** 配置文件包含未知 `options` key
- **THEN** 配置读取层将该 key/value 保留在 native options 参数来源中
- **THEN** native option 注册和 operation 适用性仍由后续 direct CLI 参数处理链路决定

#### Scenario: 高优先级配置值按来源优先级合并
- **WHEN** 调用方未显式传入 `--output`
- **AND** 项目级配置包含 `defaults.output: "readable-json"`
- **AND** 用户级配置包含 `defaults.output: "readable-view"`
- **THEN** 合并后的标准参数来源对象使用项目级 `defaults.output: "readable-json"`

#### Scenario: 配置源跳过原因作为 warning
- **WHEN** adapter direct CLI 读取到不可读、JSON 语法无效或顶层不是 JSON object 的 adapter 配置源
- **THEN** 该配置源不参与本次合并
- **THEN** SDK 产生 id 为 `adapter_config_source_skipped` 且 effect 为 `operation_continued` 的 direct CLI warning
- **THEN** warning details 包含 `source_level`、`path_origin`、`path` 和 `reason_code`
- **THEN** `source_level` 为 `project` 或 `user`
- **THEN** `path_origin` 为 `default` 或 `override`
- **THEN** `path` 为本次尝试读取的解析后路径
- **THEN** `reason_code` 为 `missing_override`、`not_file`、`unreadable`、`invalid_json` 或 `non_object`

#### Scenario: 参数来源对象交给标准参数处理链路
- **WHEN** 项目级 adapter 配置包含非法 `defaults.output`
- **AND** 调用方未显式传入 `--output`
- **THEN** SDK 将该值合并为 `output` 参数来源
- **THEN** direct CLI 复用既有 output typed validation 返回输入错误

#### Scenario: pagination 参数非法时由标准参数处理链路返回输入错误
- **WHEN** 项目级 adapter 配置包含 `defaults.pagination.limit_chars: 0`
- **OR** 项目级 adapter 配置包含非布尔 `defaults.pagination.enabled`
- **AND** 调用方未显式传入覆盖值
- **THEN** SDK 将配置源投影为标准参数来源对象
- **THEN** direct CLI 参数处理链路返回输入错误

### Requirement: Adapter invoke 不读取 direct CLI 配置
Adapter `invoke` stdin JSON MUST 保持严格 protocol input。SDK MUST NOT 在 `invoke` 路径读取项目级或用户级 adapter direct CLI 配置，也 MUST NOT 用 direct CLI 配置补全缺失的 protocol request arguments 或 adapter-owned options。schema-valid `invoke` request MUST enter the same adapter document operation handler as direct CLI document operations after request validation.

#### Scenario: Invoke 缺少参数仍按协议失败
- **WHEN** adapter `invoke` 从 stdin 收到缺少必需 `limit_chars` 的 outline request
- **AND** 项目级 adapter 配置设置了 `defaults.pagination.limit_chars`
- **THEN** SDK 按 protocol request validation 返回 `INVALID_REQUEST`
- **THEN** SDK 不从 adapter 配置补全 `limit_chars`

#### Scenario: Invoke 不读取 pagination 配置
- **WHEN** adapter `invoke` 从 stdin 收到 schema-valid outline request
- **AND** 项目级 adapter 配置设置了 `defaults.pagination.enabled: false`
- **THEN** SDK 不读取该 direct CLI 配置
- **THEN** adapter handler 只看到 stdin request 中显式携带的 `limit_chars`

#### Scenario: Invoke 不读取 native option 配置
- **WHEN** adapter `invoke` 从 stdin 收到没有 `arguments.options` 的 outline request
- **AND** 项目级 adapter 配置设置了 `options.max_heading_level`
- **THEN** SDK 不把该配置注入 request
- **THEN** adapter handler 只看到 request 中显式携带的 arguments
