本 delta 定义 adapter direct CLI 配置路径参数、配置读取、合并和 native options 显式化的共享 SDK 契约；除本 change 明确新增的配置 schema/example 参考材料外，它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不改变现有主规范或实现语义。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 支持可覆盖配置路径
`docnav-adapter-sdk` direct CLI MUST 将项目级配置文件路径和用户级配置文件路径作为 SDK-owned standard direct CLI 参数。SDK MUST 暴露 `--project-config-path <path>` 和 `--user-config-path <path>` 作为 document operation 的配置路径覆盖参数，并 MUST 在 document operation help 中展示这两个参数。SDK MUST 为两者提供默认值：项目级默认路径为项目根下 `.docnav/<adapter-id>.json`，用户级默认路径为用户配置目录下 `<adapter-id>.json`；调用方 MUST 能在配置加载前覆盖任一路径。Adapter direct CLI 的项目根 MUST 从启动 cwd 向上查找最近的 `.docnav/`，找到时使用其父目录，未找到时使用启动 cwd；document path MUST NOT 参与 adapter direct CLI 配置项目根发现。相对覆盖路径 MUST 按启动 cwd 解析。

#### Scenario: 使用默认配置路径
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md` 且未覆盖配置路径
- **THEN** SDK 使用默认项目级路径 `.docnav/docnav-markdown.json`
- **THEN** SDK 使用默认用户级路径：用户配置目录下的 `docnav-markdown.json`
- **THEN** 路径参数不进入 protocol request 或 adapter-owned options

#### Scenario: 默认项目级配置路径基于启动 cwd 发现项目根
- **WHEN** 调用方从项目子目录执行 `docnav-markdown outline docs/guide.md`
- **AND** 启动 cwd 的父级中存在最近的 `.docnav/`
- **THEN** SDK 以该 `.docnav/` 的父目录作为 adapter direct CLI 配置项目根
- **THEN** document path 不改变本次项目根发现结果

#### Scenario: 覆盖配置路径
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path fixtures/project.json --user-config-path fixtures/user.json`
- **THEN** SDK 从覆盖后的两个路径读取配置
- **THEN** 默认配置路径不参与本次配置合并
- **THEN** 覆盖路径参数不传给 operation handler

#### Scenario: 覆盖配置路径不可用
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path missing.json`
- **THEN** SDK 尝试读取覆盖后的项目级配置路径
- **THEN** 该项目级配置源不参与本次配置合并
- **THEN** direct CLI 产生配置源跳过 warning

#### Scenario: Help 展示配置路径参数
- **WHEN** 调用方执行 `docnav-markdown outline --help`
- **THEN** help 输出包含 `--project-config-path <path>` 和 `--user-config-path <path>`
- **THEN** help 不读取项目级或用户级 adapter 配置

### Requirement: Adapter document operation 使用唯一内部执行线路
`docnav-adapter-sdk` MUST 将 direct CLI argv/config 和 `invoke` stdin JSON 视为同一 adapter document operation 逻辑的不同参数来源。Document operation CLI MUST 在 request construction 前把 argv 和 adapter direct CLI config 解析为标准 operation 参数。`invoke` MUST 在进入同一 operation handler 前把 stdin JSON 校验为显式 operation 参数。SDK 和 adapter MUST NOT 为 argv/config 与 `invoke` 维护两套业务参数解释规则；入口只决定本次调用提供哪些参数。

#### Scenario: Direct CLI 参数进入同一 operation 线路
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md`
- **AND** 配置或 argv 解析出最终 `limit_chars`、`output` 和 native options
- **THEN** SDK 使用这些标准 operation 参数构造请求或调用 operation handler
- **THEN** adapter 业务逻辑不根据参数来源分叉

#### Scenario: Invoke 显式参数进入同一 operation 线路
- **WHEN** adapter `invoke` 从 stdin 收到 schema-valid outline request
- **THEN** SDK 从 request 中读取已经显式携带的 operation 参数
- **THEN** SDK 使用同一 operation handler 执行业务逻辑
- **THEN** SDK 不为 `invoke` 维护第二套默认值、native option 或配置解释规则

### Requirement: Adapter SDK direct CLI 支持自身配置域
`docnav-adapter-sdk` direct CLI MUST 支持读取解析后的项目级和用户级 adapter 配置文件。Direct CLI document operation MUST 按“显式 argv > 项目级 adapter 配置 > 用户级 adapter 配置 > 内置默认值”的优先级解析最终默认值，并 MUST 在进入 operation request construction 前合并为标准 direct CLI operation 参数。配置文件只可贡献 `defaults.limit_chars`、`defaults.output` 和当前 operation 适用的 `options`；`path`、`ref`、`query` MUST 来自 argv，`page` MUST 来自 argv 或入口固定默认 `1`。

#### Scenario: Direct CLI 使用项目级配置
- **WHEN** 项目级 `.docnav/docnav-markdown.json` 设置 `defaults.limit_chars`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md` 且未传入 `--limit-chars`
- **THEN** SDK 将项目级配置中的 limit_chars 合并到标准 direct CLI operation 参数
- **THEN** 该值在进入 operation handler 或 request construction 前已经显式化

#### Scenario: 显式 argv 覆盖配置
- **WHEN** 项目级 adapter 配置设置 `defaults.limit_chars`
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md --limit-chars 120`
- **THEN** direct CLI 使用显式 argv 值 `120`
- **THEN** 项目级和用户级配置值不覆盖显式 argv

#### Scenario: 用户级配置作为项目级缺省
- **WHEN** 用户级 `docnav-markdown.json` 设置 `defaults.output`
- **AND** 项目级配置没有设置 `defaults.output`
- **AND** 调用方未传入 `--output`
- **THEN** direct CLI 使用用户级配置中的 output 默认值

#### Scenario: 配置合并后只暴露标准 operation 参数
- **WHEN** SDK 完成 argv、项目级配置、用户级配置和内置默认值合并
- **THEN** operation request construction 只消费标准 direct CLI operation 参数
- **THEN** operation handler 不需要知道配置文件路径、配置来源或合并细节
- **THEN** 配置文件中的字段不会生成或覆盖 `path`、`ref`、`query` 或 `page`

### Requirement: Adapter SDK direct CLI 配置只产出标准参数来源对象
Adapter direct CLI config MUST 支持通用 `defaults.limit_chars`、`defaults.output` 和 `options` object。SDK MUST 按优先级把 argv、项目级配置、用户级配置和内置默认值合并为标准 direct CLI 参数来源对象。配置合并阶段 MUST 只处理配置源读取、字段映射、来源优先级和配置源跳过 warning；合并结果 MUST 表示为标准 direct CLI 参数来源对象和 direct CLI warning。配置源根值 MUST 是 JSON object。配置读取层 MUST 将 `defaults.limit_chars` 映射为 `limit_chars` 参数来源、将 `defaults.output` 映射为 `output` 参数来源、将 `options` object 映射为 native options 参数来源。生成后的参数来源对象 MUST 交给既有 direct CLI 参数处理链路完成类型、范围、枚举、native option 注册和 operation 适用性处理。

#### Scenario: defaults 字段映射为标准参数来源
- **WHEN** 配置文件包含 `defaults.limit_chars: 6000`
- **AND** 配置文件包含 `defaults.output: "readable-view"`
- **THEN** SDK 将 `defaults.limit_chars` 合并为 `limit_chars` 参数来源
- **THEN** SDK 将 `defaults.output` 合并为 `output` 参数来源

#### Scenario: 配置 options 合并为标准 native option 参数
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **THEN** SDK 将 `options` object 合并为标准 native options 参数来源
- **THEN** native option 注册、value 处理和 operation 适用性由既有 native option 处理链路决定

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

### Requirement: Adapter invoke 不读取 direct CLI 配置
Adapter `invoke` stdin JSON MUST 保持严格 protocol input。SDK MUST NOT 在 `invoke` 路径读取项目级或用户级 adapter direct CLI 配置，也 MUST NOT 用 direct CLI 配置补全缺失的 protocol request arguments 或 adapter-owned options。schema-valid `invoke` request MUST enter the same adapter document operation handler as direct CLI document operations after request validation.

#### Scenario: Invoke 缺少参数仍按协议失败
- **WHEN** adapter `invoke` 从 stdin 收到缺少必需 `limit_chars` 的 outline request
- **AND** 项目级 adapter 配置设置了 `defaults.limit_chars`
- **THEN** SDK 按 protocol request validation 返回 `INVALID_REQUEST`
- **THEN** SDK 不从 adapter 配置补全 `limit_chars`

#### Scenario: Invoke 不读取 native option 配置
- **WHEN** adapter `invoke` 从 stdin 收到没有 `arguments.options` 的 outline request
- **AND** 项目级 adapter 配置设置了 `options.max_heading_level`
- **THEN** SDK 不把该配置注入 request
- **THEN** adapter handler 只看到 request 中显式携带的 arguments
