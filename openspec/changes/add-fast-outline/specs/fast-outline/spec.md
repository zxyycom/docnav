## 一句话核心

fast outline MUST 在小文档时返回直接读取结果，在大文档或无法判断时返回 outline 结果。

## 文档状态

当前 spec 是未审计的临时需求草案。实现前必须先审计每条 requirement 是否符合主规范和协议边界；审计未完成前，不要把本文中的字段名、tool 名或输出结构当作最终规范。

## ADDED Requirements

### Requirement: fast outline 命令
`docnav` SHALL 提供 `fast-outline` 文档命令，支持 path、可选 adapter、page、limit character budget 和受支持的阅读输出模式参数。

#### Scenario: 调用 fast outline
- **WHEN** 调用方运行 `docnav fast-outline docs/guide.md`
- **THEN** `docnav` SHALL 使用与 `docnav outline` 相同的所有权规则解析项目上下文、规范化 path、选择 adapter 并应用 core 默认值

#### Scenario: 现有 outline 行为保持不变
- **WHEN** 调用方运行 `docnav outline docs/guide.md`
- **THEN** `docnav` SHALL 返回 outline 行为，并且不应用 fast outline 的直接读取选择

### Requirement: adapter 提供全文 ref
adapter SHALL 能在 outline 结果中暴露可选全文 ref；`docnav` 在评估 fast outline eligibility 时 SHALL 原样把该 ref 传给 read。

#### Scenario: adapter 提供全文 ref
- **WHEN** adapter 返回包含全文 ref 的 outline 结果
- **THEN** `docnav fast-outline` SHALL 使用该 exact ref 值构造 eligibility read 请求

#### Scenario: adapter 未提供全文 ref
- **WHEN** adapter 返回不包含全文 ref 的 outline 结果
- **THEN** `docnav fast-outline` SHALL 回退为返回 outline 结果

### Requirement: 小文档直接读取
当 adapter 提供的全文 read 在 fast outline 字符预算内读完时，`docnav fast-outline` SHALL 直接读取全文。

#### Scenario: 全文 read 可一次读完
- **WHEN** 全文 read 请求返回内容并且 `page: null`
- **THEN** `docnav fast-outline` SHALL 返回 read-mode 结果，包含 read 的 content、content_type、cost、ref 和 page 字段

#### Scenario: 全文 read 还有下一页
- **WHEN** 全文 read 请求返回非空 next page
- **THEN** `docnav fast-outline` SHALL 返回 outline-mode 结果，包含原始 outline entries 和 page 字段

### Requirement: fast outline 输出形状
fast outline 的 readable JSON 和 MCP structuredContent SHALL 显式标识结果是直接读取还是 outline 回退。

#### Scenario: readable JSON 直接读取
- **WHEN** `docnav fast-outline <path> --output readable-json` 直接读取文档
- **THEN** stdout SHALL 包含 `mode: "read"`，并包含与 readable read 相同业务字段的 read payload

#### Scenario: readable JSON outline 回退
- **WHEN** `docnav fast-outline <path> --output readable-json` 返回导航
- **THEN** stdout SHALL 包含 `mode: "outline"`，并包含与 readable outline 相同业务字段的 outline payload

#### Scenario: 默认文本直接读取
- **WHEN** `docnav fast-outline <path>` 直接读取文档
- **THEN** stdout SHALL 展示可读内容，而不是 outline entries

#### Scenario: 默认文本 outline 回退
- **WHEN** `docnav fast-outline <path>` 返回导航
- **THEN** stdout SHALL 使用现有 outline 文本约定展示 outline entries

### Requirement: MCP fast outline 映射
`docnav-mcp` SHALL 通过专用 tool 暴露 fast outline 行为，并将其映射到核心 `docnav fast-outline` 命令。

#### Scenario: MCP 调用 fast outline
- **WHEN** MCP 调用方使用 fast outline tool 并传入文档 path
- **THEN** `docnav-mcp` SHALL 调用 `docnav fast-outline`，并把 readable 结果转换为 TextContent 和 structuredContent，且不解析文档内容

#### Scenario: MCP outline 保持不变
- **WHEN** MCP 调用方使用现有 outline tool
- **THEN** `docnav-mcp` SHALL 保持现有 outline-only 映射

### Requirement: 协议兼容
fast outline SHALL NOT 引入新的 adapter `invoke` operation。

#### Scenario: adapter 调用
- **WHEN** `docnav fast-outline` 评估文档
- **THEN** adapter subprocess 调用 SHALL 只使用现有 `outline` 和 `read` operations

#### Scenario: 现有协议调用方
- **WHEN** 现有调用方发送 `outline` 或 `read` invoke 请求
- **THEN** 该请求 SHALL 继续有效，且不要求携带 fast outline 字段
