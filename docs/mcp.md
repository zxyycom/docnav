# MCP Handoff

本文是 `docnav-mcp` 目标制品、tool 映射和 MCP 阅读输出交接边界的主规范。`docnav-mcp` 当前由 `implement-docnav-mcp-bridge` change 承接实现；本文只定义 ownership 和 handoff，不把目标 bridge 描述为当前 core CLI 已交付能力。

CLI 命令面见 [CLI](cli.md)，标准参数机制见 [标准参数](standard-parameters.md)，输出模式和 readable contract 见 [输出模式](output.md)，原始协议见 [原始协议](protocol.md)。

## 目标职责

`docnav-mcp` 是 Node.js MCP bridge 的目标制品。源码以 TypeScript 维护；发布给 MCP client 的 bridge 制品是单文件 JavaScript。目标 MCP bridge 必须依赖系统中可调用的 `docnav` 核心 CLI，并消费 [输出模式](output.md) 定义的 readable output contract。

目标 MCP tools：

- `document_outline`
- `document_read`
- `document_find`
- `document_info`

MCP bridge 的目标职责是将 MCP 参数映射为核心 `docnav` CLI 调用，并将 `docnav` readable 结果转换为 MCP TextContent 和 structuredContent。目标 tools 可传入可选 `adapter` 字符串，映射到 `docnav --adapter <adapter-id>`。

## 参数映射

MCP tool input schema、tool input path、stable param identity、default metadata 和 tool input 到标准参数的映射由 [标准参数](standard-parameters.md#metadata-与交接边界) 定义。MCP bridge 可以消费 Rust generated JSON artifact、runtime metadata 或人工同步的等价 metadata，但必须保持 tool -> operation 映射、operation registration、MCP metadata 和可选 CLI argv transport metadata 一致。

当前 transport 可继续把 MCP tool input 映射为 core `docnav` CLI argv；CLI argv spelling 只是 transport metadata，不是 MCP tool input 的语义来源。MCP bridge 不构造 adapter invoke request，也不维护独立的参数 requiredness、default、range 或 enum 规则。

## 所有权边界

MCP bridge 不拥有以下职责：

- 文档解析。
- 格式识别。
- adapter 管理。
- 项目初始化。
- core 标准参数解析。
- adapter 路由和下级适配层调用。

这些职责由 `docnav`、adapter 或对应主规范拥有；MCP bridge 只做 tool call 到 core CLI 的参数映射和 readable 结果包装。

## 输出目标

MCP 输出目标属于阅读输出层：

- TextContent 是简洁、可直接阅读的结果，并保留 page 状态。
- structuredContent 使用 operation 对应的精简 readable schema，服务工具声明和客户端展示，不替代完整协议接口。
- 存在兼容性 warning 时，structuredContent 必须包含顶层 `warnings` 数组。
- structuredContent 不包含完整 invoke envelope。
- TextContent 不复制完整 protocol JSON。
- page 状态使用紧凑文本表达，例如：

```text
page: 2
```

每个 target tool 声明精简 readable `outputSchema`。工具声明中的 `outputSchema` 必须内联或随工具声明打包，不依赖远程 schema URL；独立 schema 文件仍作为文档和测试来源。

MCP bridge 源码、单文件 JavaScript 制品、TextContent bridge wiring、tool declaration 打包和 MCP error mapping 保留在 `implement-docnav-mcp-bridge` change 中实现。
