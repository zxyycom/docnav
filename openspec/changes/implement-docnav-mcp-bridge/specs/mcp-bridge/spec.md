## ADDED Requirements

### Requirement: docnav-mcp 必须通过 stdio 暴露 MCP tools
`docnav-mcp` MUST 是 Node.js/JavaScript MCP bridge，并 MUST 通过 stdio transport 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。

#### Scenario: 启动 MCP bridge
- **WHEN** MCP Client 启动 `docnav-mcp`
- **THEN** bridge 通过 stdio 提供 MCP transport
- **THEN** tool 列表包含四个 document tools

### Requirement: MCP tool call 必须映射到核心 docnav CLI
`docnav-mcp` MUST 将 tool 参数直接映射为核心 `docnav` CLI 调用，并 MUST 为每个 document tool 固定传入 `--output readable-json`。Adapter 调用、adapter 选择和格式识别 MUST 由核心 `docnav` CLI 执行。

#### Scenario: document_read 调用
- **WHEN** MCP Client 调用 `document_read` 并提供 path、ref、page 和 limit_chars
- **THEN** `docnav-mcp` 调用核心 `docnav read --output readable-json`
- **THEN** ref 原样传递给 `docnav`

#### Scenario: document_outline 调用
- **WHEN** MCP Client 调用 `document_outline` 并提供 path、page 和 limit_chars
- **THEN** `docnav-mcp` 调用核心 `docnav outline --output readable-json`
- **THEN** structuredContent 从 stdout readable JSON 解析得到

### Requirement: MCP adapter 参数必须原样映射
MCP tool 的可选 `adapter` 参数 MUST 映射为 `docnav --adapter`。Adapter id 解释、格式识别和候选继续遍历 MUST 由核心 `docnav` CLI 完成。

#### Scenario: 传递 adapter id
- **WHEN** MCP Client 传入 `adapter: "docnav-markdown"`
- **THEN** `docnav-mcp` 将其传给 `docnav --adapter docnav-markdown`
- **THEN** adapter 选择由 `docnav` 完成

### Requirement: structuredContent 必须使用 readable schema
MCP structuredContent MUST 使用对应 operation 的 readable schema，并 MUST 从 `docnav --output readable-json` 的 stdout JSON 派生。structuredContent MUST 只包含 operation readable 字段、readable 错误字段和可选 `warnings`；read structuredContent MUST 保留 `content_type`。MCP TextContent 渲染 SHOULD 消费 `replace-text-with-readable-view` 的 readable-view contract 和仓库 renderer config；block pointer、byte length 和 block payload 语义 SHOULD 与 Rust renderer 一致。

#### Scenario: document_outline structuredContent
- **WHEN** `document_outline` 返回结果
- **THEN** structuredContent 只包含 entries 和 page
- **THEN** structuredContent 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`

#### Scenario: document_read structuredContent
- **WHEN** `document_read` 返回结果
- **THEN** structuredContent 包含 ref、content、content_type、cost 和 page

#### Scenario: warning structuredContent
- **WHEN** `docnav --output readable-json` 返回包含 `warnings` 的成功结果
- **THEN** MCP structuredContent 保留 `warnings`
- **THEN** MCP TextContent 在正常阅读文本后追加 warning 文本

#### Scenario: 成功 stderr 不升级为错误
- **WHEN** `docnav` 子进程成功退出并向 stderr 写入非致命诊断
- **THEN** `docnav-mcp` 返回成功的 MCP tool result
- **THEN** 成功/失败判定以退出码和 stdout readable JSON payload 为准

### Requirement: tool outputSchema 必须内联或随包打包
`docnav-mcp` MUST 为每个 tool 声明精简 readable outputSchema，并 MUST 内联或随包打包 schema。Schema MUST 可在离线环境中使用。

#### Scenario: 声明 outputSchema
- **WHEN** MCP Client 读取 tool 声明
- **THEN** 每个 document tool 都包含对应 outputSchema
- **THEN** schema 可在离线环境中使用

### Requirement: MCP 错误输出必须保持阅读层语义
`docnav-mcp` MUST 将 `docnav` readable 错误转换为 MCP TextContent 和 structuredContent，并 MUST 保留必要 code/details。完整 protocol envelope MUST 由 `docnav --output protocol-json` 提供，而不是 MCP structuredContent 的输出形状。

#### Scenario: docnav 返回 REF_NOT_FOUND
- **WHEN** 核心 `docnav` 返回 `REF_NOT_FOUND`
- **THEN** MCP structuredContent 保留错误 code
- **THEN** 输出不包含 protocol envelope 字段
