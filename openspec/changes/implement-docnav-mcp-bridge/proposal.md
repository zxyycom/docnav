**一句话核心：实现格式无关的 `docnav-mcp` stdio bridge，只把 MCP tool call 映射到核心 `docnav` CLI。**

## Why

v0 文档要求 MCP 是接入层，而不是解析层或路由层。核心 CLI 与 Markdown 链路稳定后，需要实现 Node.js/JavaScript `docnav-mcp`，向 MCP Client 暴露四个文档工具，并从稳定 `ProtocolResponse` facts 派生 MCP structuredContent。

## What Changes

- 新增 `docnav-mcp` Node.js/JavaScript 可安装 bin，通过 stdio 提供 MCP transport。
- 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。
- 将 MCP tool 参数直接映射为核心 `docnav` CLI 参数，包括 path、ref、query、可选 `adapter`、`page` 和 `limit_chars`。
- 所有 document tool 固定调用 `docnav --output protocol-json`；bridge 校验 stdout `ProtocolResponse`，不解析默认 `readable-view` 文本。
- successful MCP structuredContent 从 `ProtocolResponse::Success.result` 映射 tool-owned 字段；失败时从 `ProtocolResponse::Failure.error` 映射 code、message、owner、guidance 和 details，不复制完整 protocol envelope。
- TextContent 由 bridge 从同一 protocol result/error facts 生成，presentation contract 归 MCP bridge；bridge 不解析 Markdown、不复制 Rust `readable-view` block framing，也不发起第二次 CLI 调用。
- 本 change 的 JavaScript TextContent renderer、MCP outputSchema 和 bridge wiring 实现任务保留在当前 change；核心 protocol envelope 和 result/error 字段继续由 `docs/protocol.md` 拥有。
- 子进程成功退出时，stderr 中的 owner-scoped status 不升级为 MCP 错误；bridge 以 protocol response、退出码和 `ok` 字段决定成功/失败。
- 内联或随包打包 MCP tool `outputSchema`，不依赖远程 schema URL。
- 边界：adapter 调用、格式识别、adapter 管理和 Markdown 解析由核心 CLI 或 adapter 所属 change 负责；本 change 只实现 MCP 接入层。

## Capabilities

### New Capabilities

- `mcp-bridge`: 定义 MCP stdio bridge、四个 document tools、CLI 参数映射、TextContent/structuredContent 输出和 tool schema 声明。

### Modified Capabilities

- 无。

## Impact

- 影响 MCP 接入制品：`docnav-mcp`。
- 影响 Node.js/JavaScript 包装、tool schema 打包和 CLI 子进程调用。
- 影响端到端测试：MCP tool call 到 `docnav` CLI 的映射、structuredContent schema 校验和 protocol envelope 排除。
- 依赖 contract：本 change 消费当前 `protocol-json` envelope、operation result 和 protocol error contract；MCP-specific structuredContent/TextContent 由本 change 自己定义和验证。
