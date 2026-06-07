**一句话核心：实现格式无关的 `docnav-mcp` stdio bridge，只把 MCP tool call 映射到核心 `docnav` CLI。**

## Why

v0 文档要求 MCP 是接入层，而不是解析层或路由层。核心 CLI 与 Markdown 链路稳定后，需要实现 Node.js/JavaScript `docnav-mcp`，向 MCP Client 暴露四个文档工具，并保持 structuredContent 与 readable schema 一致。

## What Changes

- 新增 `docnav-mcp` Node.js/JavaScript 可安装 bin，通过 stdio 提供 MCP transport。
- 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。
- 将 MCP tool 参数直接映射为核心 `docnav` CLI 参数，包括 path、ref、query、可选 `adapter`、`page` 和 `limit_chars`。
- 所有 document tool 固定调用 `docnav --output readable-json` 获取结构化阅读结果；structuredContent 来自 readable JSON，不解析默认人类文本。
- 将 `docnav` 返回的 warnings 映射到 MCP 输出：TextContent 在正常阅读文本后追加 warning 文本，structuredContent 保留 `warnings` 字段。
- 子进程成功退出时，stderr 中的非致命 warning 或诊断不升级为 MCP 错误；bridge 以 readable JSON payload 和退出码决定成功/失败。
- 内联或随包打包 MCP tool `outputSchema`，不依赖远程 schema URL。
- 边界：adapter 调用、格式识别、adapter 管理和 Markdown 解析由核心 CLI 或 adapter 所属 change 负责；本 change 只实现 MCP 接入层。

## Capabilities

### New Capabilities

- `docnav-mcp-bridge-implementation`: 实现 MCP stdio bridge、四个 document tools、CLI 参数映射、TextContent/structuredContent 输出和 tool schema 声明。

### Modified Capabilities

- 无。

## Impact

- 影响 MCP 接入制品：`docnav-mcp`。
- 影响 Node.js/JavaScript 包装、tool schema 打包和 CLI 子进程调用。
- 影响端到端测试：MCP tool call 到 `docnav` CLI 的映射、structuredContent schema 校验和 protocol envelope 排除。
