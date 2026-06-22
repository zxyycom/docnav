本 delta 补强 Docnav 共享契约中的 CLI 配置域隔离和 adapter direct CLI 配置边界；它只在 `openspec/changes/implement-adapter-direct-cli-config/` 下形成未审核临时文档，不影响现有其它文档或主规范。

## ADDED Requirements

### Requirement: Adapter direct CLI 配置域不得被 core 或 MCP 重新解释
每个 adapter direct CLI MUST 只读取自身 adapter id 对应的配置域，并 MAY 暴露由 SDK 拥有的 `--project-config-path <path>` 和 `--user-config-path <path>` 以覆盖项目级和用户级 adapter 配置文件路径。Core `docnav` 和 `docnav-mcp` MUST NOT 读取 adapter direct CLI 配置，MUST NOT 从 adapter direct CLI 配置合成格式专属 `arguments.options`，并 MUST 继续只通过 protocol/request/readable output 边界与 adapter 交互。

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

### Requirement: Direct CLI 配置错误必须可诊断
Adapter direct CLI document operation MUST 对自身配置文件缺失、JSON 解析失败、未知 key、非法值和 native option 校验失败提供可诊断错误。缺失配置文件 MUST 表示对应层没有配置值，不得作为错误。配置错误 MUST 按已确定的 document output mode 渲染：`protocol-json` 使用 protocol failure envelope，`readable-json` 使用 readable error JSON，默认 document output 使用 readable-view error framing。

#### Scenario: 缺失配置文件使用下一级默认值
- **WHEN** 项目级或用户级 adapter 配置文件不存在
- **THEN** adapter direct CLI 继续按其余来源解析默认值
- **THEN** 缺失文件不产生 warning 或错误

#### Scenario: 配置文件非法时阻断 document operation
- **WHEN** adapter direct CLI 读取到语法非法的 JSON 配置文件
- **THEN** document operation 返回非零退出
- **THEN** 错误 payload 或诊断包含配置文件路径和失败原因
- **THEN** 不执行 adapter operation handler

#### Scenario: readable-json 配置错误保持 readable error shape
- **WHEN** adapter direct CLI 读取到非法配置文件
- **AND** 调用方显式传入 `--output readable-json`
- **THEN** stdout 输出 readable error JSON
- **THEN** readable error 保留错误 code 和配置路径或配置 key details
- **THEN** stderr 不替代 stdout 的 readable error payload
