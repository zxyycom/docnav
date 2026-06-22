本 delta 定义 adapter direct CLI 配置路径参数、配置读取、合并和 native options 显式化的共享 SDK 契约；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter SDK direct CLI 支持可覆盖配置路径
`docnav-adapter-sdk` direct CLI MUST 将项目级配置文件路径和用户级配置文件路径作为 SDK-owned standard direct CLI 参数。SDK MUST 暴露 `--project-config-path <path>` 和 `--user-config-path <path>` 作为 document operation 的配置路径覆盖参数。SDK MUST 为两者提供默认值：项目级默认路径为项目根下 `.docnav/<adapter-id>.json`，用户级默认路径为用户配置目录下 `<adapter-id>.json`；调用方 MUST 能在配置加载前覆盖任一路径。相对覆盖路径 MUST 按启动 cwd 解析。

#### Scenario: 使用默认配置路径
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md` 且未覆盖配置路径
- **THEN** SDK 使用默认项目级路径 `.docnav/docnav-markdown.json`
- **THEN** SDK 使用默认用户级路径：用户配置目录下的 `docnav-markdown.json`
- **THEN** 路径参数不进入 protocol request 或 adapter-owned options

#### Scenario: 覆盖配置路径
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path fixtures/project.json --user-config-path fixtures/user.json`
- **THEN** SDK 从覆盖后的两个路径读取配置
- **THEN** 默认配置路径不参与本次配置合并
- **THEN** 覆盖路径参数不传给 operation handler

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

### Requirement: Adapter SDK direct CLI 配置必须校验通用默认值和 native options
Adapter direct CLI config MUST 支持通用 `defaults.limit_chars`、`defaults.output` 和 `options` object。`defaults.limit_chars` MUST 是正整数；`defaults.output` MUST 是 document output mode；`options` 中的 key MUST 来自 adapter 注册的 `NativeOptionSpec`，value MUST 通过对应 native option validation。

#### Scenario: 配置中的 native option 写入 options
- **WHEN** `docnav-markdown` 注册 native option `max_heading_level`
- **AND** 配置文件包含 `options.max_heading_level: 2`
- **AND** 调用方执行适用该 option 的 `outline`
- **THEN** direct CLI 在最终 operation input 中写入 adapter-owned option `max_heading_level: 2`

#### Scenario: 不适用 operation 的 native option 不注入
- **WHEN** 配置文件包含 `options.max_heading_level: 2`
- **AND** 调用方执行不适用该 option 的 `read`
- **THEN** direct CLI 不把 `max_heading_level` 写入 read 的 options
- **THEN** read 的必需参数和通用默认值仍按配置优先级解析

#### Scenario: 配置非法时失败
- **WHEN** adapter direct CLI document operation 读取到未知配置 key、非法 output、非正整数 limit_chars 或非法 native option value
- **THEN** direct CLI 返回输入/config 错误，并使用 `INVALID_REQUEST` 表示 protocol-json failure
- **THEN** document operation handler 不执行

#### Scenario: protocol-json 配置错误保持协议 envelope
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --output protocol-json`
- **AND** 项目级配置文件包含非法 `defaults.limit_chars`
- **THEN** stdout 输出 protocol failure envelope
- **THEN** failure 使用 `INVALID_REQUEST`
- **THEN** `details.field` 指向非法配置 key
- **THEN** stderr 不承载替代 stdout envelope 的协议外错误

### Requirement: Adapter invoke 不读取 direct CLI 配置
Adapter `invoke` stdin JSON MUST 保持严格 protocol input。SDK MUST NOT 在 `invoke` 路径读取项目级或用户级 adapter direct CLI 配置，也 MUST NOT 用 direct CLI 配置补全缺失的 protocol request arguments 或 adapter-owned options。

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
