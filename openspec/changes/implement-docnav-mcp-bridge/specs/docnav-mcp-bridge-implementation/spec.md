## ADDED Requirements

### Requirement: docnav-mcp 必须通过 stdio 暴露 MCP tools
`docnav-mcp` MUST 是 Node.js/JavaScript MCP bridge，并 MUST 通过 stdio transport 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。

#### Scenario: 启动 MCP bridge
- **WHEN** MCP Client 启动 `docnav-mcp`
- **THEN** bridge 通过 stdio 提供 MCP transport
- **THEN** tool 列表包含四个 document tools

### Requirement: MCP tool call 必须映射到核心 docnav CLI
`docnav-mcp` MUST 将 tool 参数直接映射为核心 `docnav` CLI 调用，MUST NOT 直接调用 adapter 或执行格式识别。

#### Scenario: document_read 调用
- **WHEN** MCP Client 调用 `document_read` 并提供 path、ref、page 和 limit_chars
- **THEN** `docnav-mcp` 调用核心 `docnav read`
- **THEN** ref 原样传递给 `docnav`

### Requirement: MCP format 参数必须原样映射
MCP tool 的可选 `format` 参数 MUST 映射为 `docnav --format`，`docnav-mcp` MUST NOT 自行解释 format id 或 content type。

#### Scenario: 传递 content type
- **WHEN** MCP Client 传入 `format: "text/markdown"`
- **THEN** `docnav-mcp` 将其传给 `docnav --format text/markdown`
- **THEN** adapter 选择由 `docnav` 完成

### Requirement: structuredContent 必须使用 readable schema
MCP structuredContent MUST 使用对应 operation 的 readable schema，MUST NOT 包含 protocol envelope 字段，且 read structuredContent MUST 保留 `content_type`。

#### Scenario: document_outline structuredContent
- **WHEN** `document_outline` 返回结果
- **THEN** structuredContent 只包含 entries 和 page
- **THEN** structuredContent 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`

#### Scenario: document_read structuredContent
- **WHEN** `document_read` 返回结果
- **THEN** structuredContent 包含 ref、content、content_type、cost 和 page

### Requirement: tool outputSchema 必须内联或随包打包
`docnav-mcp` MUST 为每个 tool 声明精简 readable outputSchema，并 MUST 内联或随包打包 schema，MUST NOT 依赖远程 schema URL。

#### Scenario: 声明 outputSchema
- **WHEN** MCP Client 读取 tool 声明
- **THEN** 每个 document tool 都包含对应 outputSchema
- **THEN** schema 可在离线环境中使用

### Requirement: MCP 错误输出必须保持阅读层语义
`docnav-mcp` MUST 将 `docnav` readable 错误转换为 MCP TextContent 和 structuredContent，并 MUST 保留必要 code/details，MUST NOT 暴露完整 protocol envelope。

#### Scenario: docnav 返回 REF_NOT_FOUND
- **WHEN** 核心 `docnav` 返回 `REF_NOT_FOUND`
- **THEN** MCP structuredContent 保留错误 code
- **THEN** 输出不包含 protocol envelope 字段
