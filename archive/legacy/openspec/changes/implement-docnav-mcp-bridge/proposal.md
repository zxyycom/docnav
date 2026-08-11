**一句话核心：未来的 `docnav-mcp` 是格式无关 stdio bridge，只把 MCP tool call 映射到核心 `docnav` CLI。**

## 文档状态

- 状态：`product-deferred`，即 MCP bridge 尚未进入当前实施排序；长期产品方向由 [核心契约稳定后再扩张接入与交互面](../../../../../docs/decisions/product-direction/stabilize-core-before-entrypoint-expansion.md)拥有。
- 当前允许：维护探索和 target artifacts、修正与 Current owner 的失配；artifact 完整、审计通过或实现可行都不构成实施授权。
- 恢复门禁：先明确批准 MCP bridge 进入当前产品排序，再从届时 adapter lifecycle、find、protocol 和 output 的 Current 基线重新审计；执行顺序见 [tasks](tasks.md)。

## Why

MCP 的长期边界是接入层，而不是解析层或路由层。基础文档契约稳定并重新批准产品时机后，目标是实现 Node.js/JavaScript `docnav-mcp`，向 MCP Client 暴露四个文档工具，并从稳定 `ProtocolResponse` facts 派生 MCP structuredContent。

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
