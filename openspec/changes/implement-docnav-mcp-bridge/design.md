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

- Markdown 和其它格式内容由 adapter 解析；MCP bridge 只处理 tool 输入输出包装。
- Adapter probe、格式识别和 adapter 选择由核心 `docnav` CLI 执行。
- Adapter 安装管理由 adapter 管理 change 实现。
- MCP structuredContent 使用 readable schema；protocol envelope 通过 `docnav --output protocol-json` 供机器稳定解析。

## Decisions

1. MCP bridge 通过子进程调用 `docnav`。
   - 理由：adapter 选择、配置解析和错误映射属于核心 CLI。
   - 边界：MCP bridge 不直接调用 adapter；所有 document tool 都构造 `docnav <operation> ... --output readable-json`。

2. structuredContent 使用 readable schema。
   - 每个 tool 声明对应 operation 的精简 outputSchema。
   - successful structuredContent 只映射 readable success payload 字段，不包含 `protocol_version`、`request_id`、`operation` 或 `ok`。
   - structuredContent 从 `docnav --output readable-json` 的 stdout 解析得到，不解析默认人类文本。

3. TextContent 承载精简阅读文本。
   - TextContent 文本渲染消费 `replace-text-with-readable-view` 的 readable-view contract、仓库 renderer config 和 conformance vectors。
   - Bridge 从核心 CLI readable output 获得结构化结果；Markdown parsing 和 block 字段选择继续由 owning layer 负责。
   - 机器稳定解析仍必须使用 `docnav --output protocol-json`。

4. MCP adapter 参数原样映射为 `docnav --adapter`。
   - MCP bridge 不解释 adapter id，不执行格式识别。
   - 失败处理和候选继续遍历由核心 CLI 完成。

5. 错误返回保留阅读语义。
   - MCP structuredContent 和 TextContent 映射 readable error 中的 primary `DiagnosticRecord`，至少保留 code/message/owner，并在存在时保留 guidance/details。
   - 不复制完整 protocol 错误 envelope。
   - 子进程退出码为 0 时，stderr 中的 owner-scoped status 不自动变成 MCP 错误；structuredContent 仍只来自 stdout readable JSON。

## Risks / Trade-offs

- [子进程调用开销] → v0 优先保证职责边界和一致性，性能优化后续评估。
- [schema 打包漂移] → tool outputSchema 从仓库 schema 生成或同步验证，禁止依赖远程 URL。
- [MCP 文本模板影响字段] → 配置只能影响 TextContent 文案，不改变 structuredContent shape。
- [stderr status 被误判为失败] → 以 `docnav` 退出码和 stdout readable JSON payload 为准；成功退出时 stderr 非空不升级为 MCP 错误。

## Migration Plan

1. 在核心 CLI 输出稳定后实现 MCP bridge。
2. 先完成 tool 声明和参数映射，再接入 TextContent 和 structuredContent。
3. 用端到端 fixture 验证 MCP 与 CLI readable 业务语义一致。

## Open Questions

无未回答 contract 开放问题。MCP SDK 具体版本在实现时按当前 Node.js 生态选择，但 tool schema 和 stdio 行为必须满足主规范。
