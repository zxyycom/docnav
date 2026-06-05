**一句话核心：MCP bridge 是薄接入层，只做 tool 声明、参数映射和阅读输出包装。**

## Context

`docnav-mcp` 必须使用 Node.js/JavaScript，通过 stdio 暴露 MCP transport。它依赖系统可调用的 `docnav` 核心 CLI，不直接调用 adapter，也不拥有格式识别和 adapter 管理职责。

## Goals / Non-Goals

**Goals:**

- 实现 npm 可安装的 `docnav-mcp` bin。
- 暴露 `document_outline`、`document_read`、`document_find` 和 `document_info`。
- 将 MCP 参数直接映射为 `docnav` CLI 参数。
- 将 `docnav` readable 输出转换为 MCP TextContent 和 structuredContent。
- 打包或内联 tool outputSchema。

**Non-Goals:**

- 不解析 Markdown 或其它格式内容。
- 不执行 adapter probe 或 invoke。
- 不实现 adapter 安装管理。
- 不把 protocol envelope 暴露到 MCP structuredContent。

## Decisions

1. MCP bridge 通过子进程调用 `docnav`。
   - 理由：adapter 选择、配置解析和错误映射属于核心 CLI。
   - 替代方案：MCP 直接调用 adapter；拒绝，因为会复制路由逻辑并破坏架构边界。

2. structuredContent 使用 readable schema。
   - 每个 tool 声明对应 operation 的精简 outputSchema。
   - structuredContent 不包含 `protocol_version`、`request_id`、`operation` 或 `ok`。

3. TextContent 只承载精简阅读文本。
   - 文本模板可由 MCP 配置域影响。
   - 机器稳定解析仍必须使用 `docnav --output protocol-json` 或 adapter invoke。

4. MCP format 参数原样映射为 `docnav --format`。
   - MCP bridge 不解释 format id 或 content type。
   - 失败和 fallback 由核心 CLI 完成。

5. 错误返回保留阅读语义。
   - MCP structuredContent 保留必要 code/details。
   - 不复制完整 protocol 错误 envelope。

## Risks / Trade-offs

- [子进程调用开销] → v0 优先保证职责边界和一致性，性能优化后续评估。
- [schema 打包漂移] → tool outputSchema 从仓库 schema 生成或同步验证，禁止依赖远程 URL。
- [MCP 文本模板影响字段] → 配置只能影响 TextContent 文案，不改变 structuredContent shape。

## Migration Plan

1. 在核心 CLI 输出稳定后实现 MCP bridge。
2. 先完成 tool 声明和参数映射，再接入 TextContent 和 structuredContent。
3. 用端到端 fixture 验证 MCP 与 CLI readable 业务语义一致。

## Open Questions

- MCP SDK 具体版本在实现时按当前 Node.js 生态选择，但 tool schema 和 stdio 行为必须满足主规范。
