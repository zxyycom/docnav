本 delta 是该 change 的正式 OpenSpec delta，补强 Docnav 共享契约中的 CLI 配置域隔离和 adapter direct CLI 配置边界；实现阶段按 tasks 同步 owner 主规范、代码、schema/example 和测试。

## ADDED Requirements

### Requirement: Adapter direct CLI 配置域不得被 core 或 MCP 重新解释
每个 adapter direct CLI MUST 只读取自身 adapter id 对应的配置域，并 MAY 暴露由 SDK 拥有的 `--project-config-path <path>` 和 `--user-config-path <path>` 以覆盖项目级和用户级 adapter 配置文件路径。Core `docnav` 和 `docnav-mcp` MUST NOT 读取 adapter direct CLI 配置，MUST NOT 从 adapter direct CLI 配置合成格式专属 `arguments.options`，并 MUST 继续只通过 protocol/request/readable output 边界与 adapter 交互。CLI argv、adapter `invoke` stdin JSON 和 MCP tool arguments 都是入口传参方式；对应 CLI 内部 document operation 线路 MUST 保持业务逻辑唯一来源。

#### Scenario: Core 不读取 Markdown adapter 配置
- **WHEN** `.docnav/docnav-markdown.json` 设置 `options.max_heading_level`
- **AND** 调用方执行 core `docnav outline docs/guide.md`
- **THEN** core `docnav` 不读取该 adapter 配置文件
- **THEN** core `docnav` 不从该配置合成 Markdown `arguments.options`
- **THEN** adapter-specific options 只有在请求中显式存在时才传给 adapter

#### Scenario: 配置路径覆盖只属于 adapter direct CLI
- **WHEN** 调用方执行 `docnav-markdown outline docs/guide.md --project-config-path fixtures/project.json`
- **THEN** 该路径覆盖只影响本次 adapter direct CLI 配置加载
- **THEN** 路径覆盖不成为 protocol request 字段
- **THEN** core `docnav` 和 MCP 不解释该路径覆盖

#### Scenario: MCP 不读取 adapter 配置
- **WHEN** MCP client 调用 `document_outline`
- **THEN** `docnav-mcp` 只映射到 core `docnav` CLI
- **THEN** `docnav-mcp` 不读取 `.docnav/docnav-markdown.json`
- **THEN** `docnav-mcp` 不解析 adapter native options

### Requirement: Direct CLI 配置读取只提供标准参数来源对象
Adapter direct CLI document operation MUST 读取自身配置源，并把可用配置值合并为标准 direct CLI 参数来源对象。未覆盖的默认配置文件缺失表示对应层没有配置源。显式覆盖配置路径不存在、不可读或不是可读取文件时，该配置源 MUST 不参与本次合并，并 MUST 产生 direct CLI warning。JSON 语法无效或顶层不是 JSON object 的配置源 MUST 不参与本次合并，并 MUST 产生 direct CLI warning。已成功读取的配置内容 MUST 先合并为标准 direct CLI 参数来源对象，再交给既有 direct CLI 参数处理链路完成标准化和校验。配置读取层只负责 JSON 读取、固定字段投影和来源优先级；未知顶层字段或未知 `defaults` 字段不产生配置读取 warning，`options` object 内的 key/value 作为 native options 参数来源交给后续链路处理。Config schema/example MAY 作为填写提示、文档校验或 adapter package 打包材料，但 MUST NOT 成为 adapter direct CLI runtime 读取配置的前置条件。

#### Scenario: 未覆盖默认配置文件缺失时使用下一级默认值
- **WHEN** 项目级或用户级 adapter 配置文件不存在
- **AND** 调用方没有显式覆盖该层配置路径
- **THEN** adapter direct CLI 继续按其余来源解析默认值
- **THEN** 缺失文件不产生配置源输入

#### Scenario: 显式覆盖配置路径缺失时跳过该配置源
- **WHEN** 调用方显式传入 `--project-config-path missing.json`
- **AND** 该路径不存在或不可读
- **THEN** 项目级配置源不参与本次合并
- **THEN** 用户级配置和内置默认值仍按优先级参与标准参数来源对象合并
- **THEN** direct CLI 输出配置源跳过 warning

#### Scenario: Config schema 不作为 runtime gate
- **WHEN** `docs/schemas/docnav-markdown-config.schema.json` 不存在于运行环境
- **AND** 调用方执行 `docnav-markdown outline docs/guide.md`
- **THEN** adapter direct CLI 仍按配置源读取、字段投影和标准参数处理链路处理本次调用
- **THEN** runtime 不要求先加载 config schema

#### Scenario: 未知配置字段不属于配置读取错误
- **WHEN** adapter direct CLI 读取到 schema 未声明但 JSON 语法有效且顶层为 object 的配置字段
- **THEN** adapter direct CLI 不把该字段视为配置源读取失败
- **THEN** 已知字段仍按固定投影和来源优先级参与标准 direct CLI 参数来源对象合并
- **THEN** `options` object 内的 key/value 是否可用由后续 direct CLI 参数处理链路决定

#### Scenario: JSON 语法无效时继续合并其它来源
- **WHEN** adapter direct CLI 读取到语法无效的 JSON 配置文件
- **AND** 用户级 adapter 配置包含 `defaults.output: "readable-json"`
- **AND** 调用方未显式传入 `--output`
- **THEN** direct CLI 跳过项目级配置源并继续合并用户级配置
- **THEN** 合并后的标准参数来源对象使用用户级 `defaults.output`
- **THEN** direct CLI 输出配置源跳过 warning
