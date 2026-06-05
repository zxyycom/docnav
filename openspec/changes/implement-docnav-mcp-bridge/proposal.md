**一句话核心：实现格式无关的 `docnav-mcp` stdio bridge，只把 MCP tool call 映射到核心 `docnav` CLI。**

## Why

v0 文档要求 MCP 是接入层，而不是解析层或路由层。核心 CLI 与 Markdown 链路稳定后，需要实现 Node.js/JavaScript `docnav-mcp`，向 MCP Client 暴露四个文档工具，并保持 structuredContent 与 readable schema 一致。

## What Changes

- 新增 `docnav-mcp` Node.js/JavaScript 可安装 bin，通过 stdio 提供 MCP transport。
- 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。
- 将 MCP tool 参数直接映射为核心 `docnav` CLI 参数，包括可选 `format`、`page` 和 `limit_chars`。
- 将 `docnav` readable 结果转换为 MCP TextContent 和 structuredContent。
- 内联或随包打包 MCP tool `outputSchema`，不依赖远程 schema URL。
- 非目标：本 change 不直接调用 adapter、不实现格式识别、不管理 adapter、不复制 Markdown 解析逻辑。

## Capabilities

### New Capabilities

- `docnav-mcp-bridge-implementation`: 实现 MCP stdio bridge、四个 document tools、CLI 参数映射、TextContent/structuredContent 输出和 tool schema 声明。

### Modified Capabilities

- 无。

## Impact

- 影响 MCP 接入制品：`docnav-mcp`。
- 影响 Node.js/JavaScript 包装、tool schema 打包和 CLI 子进程调用。
- 影响端到端测试：MCP tool call 到 `docnav` CLI 的映射、structuredContent schema 校验和 protocol envelope 排除。
